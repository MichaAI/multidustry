use std::{
    any::{Any, TypeId},
    marker::PhantomData,
    sync::{Arc, OnceLock},
};

use async_trait::async_trait;
use dashmap::DashMap;
use kanal::SendError as KanalSendError;
use serde::{Serialize, de::DeserializeOwned};
use thiserror::Error;
use uuid::Uuid;

mod inproc;
mod transport_builder;

static LOCAL_REGISTRY: OnceLock<DashMap<Uuid, Arc<dyn DynEndpoint>>> = OnceLock::new();
pub fn get_local_registry() -> &'static DashMap<Uuid, Arc<dyn DynEndpoint + 'static>> {
    LOCAL_REGISTRY.get_or_init(|| DashMap::new())
}

pub trait Needed: Send + Serialize + DeserializeOwned + Reflectionable {}

pub trait Reflectionable {
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
pub struct TypedTx<Req: 'static + Send> {
    inner: Arc<dyn DynEndpoint>,
    _pd: PhantomData<Req>,
}

#[derive(Clone)]
pub struct TypedRx<Res: 'static + Send> {
    inner: Arc<dyn DynEndpoint>,
    _pd: PhantomData<Res>,
}

impl<Req: 'static + Needed> TypedTx<Req> {
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

impl<Res: 'static + Needed> TypedRx<Res> {
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
