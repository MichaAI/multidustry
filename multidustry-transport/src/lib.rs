use std::{
    any::Any,
    marker::PhantomData,
    sync::{Arc, OnceLock},
};

use async_trait::async_trait;
use dashmap::DashMap;
use kanal::{ReceiveError, SendError as KanalSendError};
use serde::{Serialize, de::DeserializeOwned};
use thiserror::Error;
use uuid::Uuid;

mod inproc;
mod transport_builder;

static LOCAL_REGISTRY: OnceLock<DashMap<Uuid, Arc<dyn DynEndpoint>>> = OnceLock::new();
pub fn get_local_registry() -> &'static DashMap<Uuid, Arc<dyn DynEndpoint + 'static>> {
    LOCAL_REGISTRY.get_or_init(|| DashMap::new())
}

pub trait Reflectionable: Send + Serialize + DeserializeOwned {
    fn stable_type_hash() -> &'static str;
}

#[async_trait]
trait DynEndpoint: Send + Sync {
    fn req_type(&self) -> &'static str;
    fn res_type(&self) -> &'static str;

    // Принимаем и возвращаем Box<dyn Any + Send> - это работает!
    async fn send_boxed(&self, msg: Box<dyn Any + Send>) -> Result<(), SendError>;
    async fn recv_boxed(&self) -> Result<Box<dyn Any + Send>, RecvError>;
}

#[derive(Clone)]
pub struct TypedTx<Req: Send> {
    inner: Arc<dyn DynEndpoint>,
    _pd: PhantomData<Req>,
}

#[derive(Clone)]
pub struct TypedRx<Res: Send> {
    inner: Arc<dyn DynEndpoint>,
    _pd: PhantomData<Res>,
}

impl<Req: 'static + Reflectionable> TypedTx<Req> {
    pub fn new(inner: Arc<dyn DynEndpoint>) -> Result<Self, TypeMismatchError> {
        if inner.req_type() != Req::stable_type_hash() {
            return Err(TypeMismatchError);
        }
        Ok(Self {
            inner,
            _pd: PhantomData,
        })
    }
    pub async fn send(&self, msg: Req) -> Result<(), SendError> {
        self.inner.send_boxed(Box::new(msg)).await
    }
    // опционально: pub async fn close(&self) { ... }
}

impl<Res: 'static + Reflectionable> TypedRx<Res> {
    pub fn new(inner: Arc<dyn DynEndpoint>) -> Result<Self, TypeMismatchError> {
        if inner.res_type() != Res::stable_type_hash() {
            return Err(TypeMismatchError);
        }
        Ok(Self {
            inner,
            _pd: PhantomData,
        })
    }
    pub async fn recv(&self) -> Result<Res, RecvError> {
        let any = self.inner.recv_boxed().await?;
        any.downcast::<Res>()
            .map(|b| *b)
            .map_err(|_| RecvError::TypeMismatch)
    }
}

pub struct Connection<Req: Reflectionable, Res: Reflectionable> {
    tx: TypedTx<Req>,
    rx: TypedRx<Res>,
}

impl<Req: Reflectionable, Res: Reflectionable> Connection<Req, Res> {
    fn split(self) -> (TypedTx<Req>, TypedRx<Res>) {
        (self.tx, self.rx)
    }
}

// Listener ждет соединений
pub struct Listener<Req: Reflectionable, Res: Reflectionable> {
    uuid: Uuid,
    // Канал через который клиенты сигналят о подключении
    accept_rx: kanal::AsyncReceiver<Arc<dyn DynEndpoint>>,
    _phantom: PhantomData<(Req, Res)>,
}

impl<Req: 'static + Reflectionable, Res: 'static + Reflectionable> Listener<Req, Res> {
    pub async fn accept(&self) -> Result<Connection<Req, Res>, AcceptError> {
        // Ждем соединение от клиента
        let endpoint = self.accept_rx.recv().await?;

        // Проверяем типы
        let tx = TypedTx::new(endpoint.clone())?;
        let rx = TypedRx::new(endpoint)?;

        Ok(Connection { tx, rx })
    }
}

#[derive(Debug, Error)]
pub enum SendError {
    #[error("Channel broken")]
    ChannelBroken(#[from] KanalSendError),
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

#[derive(Debug, Error)]
#[error("Type mismatch during endpoint attachment")]
pub struct TypeMismatchError;

#[derive(Error, Debug)]
pub enum AcceptError {
    #[error("Type mismatch")]
    TypeMismatch(#[from] TypeMismatchError),
    #[error("Falied to recive")]
    ReciveError(#[from] ReceiveError),
}
