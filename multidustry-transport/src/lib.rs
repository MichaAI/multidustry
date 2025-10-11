use std::marker::PhantomData;
use std::sync::OnceLock;

use async_trait::async_trait;
use dashmap::DashMap;
use serde::{Serialize, de::DeserializeOwned};
use std::any::Any;
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

pub mod inproc;
pub mod transport_builder;

pub trait Reflectionable: Send + Serialize + DeserializeOwned {
    fn stable_type_hash() -> &'static str;
}

// Для type erasure - endpoint который можем хранить в Any
#[async_trait]
trait DynEndpoint: Send + Sync {
    async fn send_boxed(&self, msg: Box<dyn Any + Send>) -> Result<(), SendError>;
    async fn recv_boxed(&self) -> Result<Box<dyn Any + Send>, RecvError>;
}

// Handle который передаём через registry
struct ListenerHandle {
    // Просто отправляем Arc<dyn DynEndpoint> в listener
    accept_tx: kanal::AsyncSender<Arc<dyn DynEndpoint>>,
}

// Локальный registry
static LOCAL_REGISTRY: OnceLock<DashMap<(Uuid, &'static str, &'static str), ListenerHandle>> =
    OnceLock::new();

fn get_local_registry() -> &'static DashMap<(Uuid, &'static str, &'static str), ListenerHandle> {
    LOCAL_REGISTRY.get_or_init(|| DashMap::new())
}

// Connection - это то что получает пользователь
pub struct Connection<Req: Reflectionable, Res: Reflectionable> {
    endpoint: Arc<dyn DynEndpoint>,
    _phantom: PhantomData<(Req, Res)>,
}

impl<Req: 'static + Reflectionable, Res: 'static + Reflectionable> Connection<Req, Res> {
    pub fn split(self) -> (TypedTx<Req>, TypedRx<Res>) {
        let tx = TypedTx {
            endpoint: self.endpoint.clone(),
            _phantom: PhantomData,
        };
        let rx = TypedRx {
            endpoint: self.endpoint,
            _phantom: PhantomData,
        };
        (tx, rx)
    }
}

// TypedTx - для отправки
pub struct TypedTx<Req: Reflectionable> {
    endpoint: Arc<dyn DynEndpoint>,
    _phantom: PhantomData<Req>,
}

impl<Req: 'static + Reflectionable> TypedTx<Req> {
    pub async fn send(&self, msg: Req) -> Result<(), SendError> {
        self.endpoint.send_boxed(Box::new(msg)).await
    }
}

// TypedRx - для получения
pub struct TypedRx<Res: Reflectionable> {
    endpoint: Arc<dyn DynEndpoint>,
    _phantom: PhantomData<Res>,
}

impl<Res: 'static + Reflectionable> TypedRx<Res> {
    pub async fn recv(&self) -> Result<Res, RecvError> {
        let any = self.endpoint.recv_boxed().await?;
        any.downcast::<Res>()
            .map(|b| *b)
            .map_err(|_| RecvError::TypeMismatch)
    }
}

// Listener - ждёт подключений
pub struct Listener<Req: Reflectionable, Res: Reflectionable> {
    accept_rx: kanal::AsyncReceiver<Arc<dyn DynEndpoint>>,
    _phantom: PhantomData<(Req, Res)>,
}

impl<Req: 'static + Reflectionable, Res: 'static + Reflectionable> Listener<Req, Res> {
    pub async fn accept(&self) -> Result<Connection<Res, Req>, RecvError> {
        let endpoint = self
            .accept_rx
            .recv()
            .await
            .map_err(|_| RecvError::ChannelClosed)?;

        Ok(Connection {
            endpoint,
            _phantom: PhantomData,
        })
    }
}

#[derive(Debug, Error)]
pub enum SendError {
    #[error("Channel closed")]
    ChannelClosed,
    #[error("Type mismatch")]
    TypeMismatch,
}

#[derive(Debug, Error)]
pub enum RecvError {
    #[error("Channel closed")]
    ChannelClosed,
    #[error("Type mismatch")]
    TypeMismatch,
}
