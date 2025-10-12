use std::{marker::PhantomData, sync::Arc, time::Duration};

use bon::Builder;
use thiserror::Error;
use uuid::Uuid;

use crate::{
    Connection, DynEndpoint, Listener, ListenerHandle, Reflectionable,
    error_strategy::ErrorStrategy, get_local_registry, guarantees::Guarantees,
    inproc::InprocEndpoint,
};

#[derive(Builder)]
pub struct TransportServer<Req: Reflectionable, Res: Reflectionable> {
    uuid: Uuid,
    #[builder(default)]
    _phantom: PhantomData<(Req, Res)>,
}

impl<Req: 'static + Reflectionable, Res: 'static + Reflectionable> TransportServer<Req, Res> {
    pub async fn create(self) -> Listener<Req, Res> {
        // Создаём канал для accept
        let (accept_tx, accept_rx) = kanal::unbounded_async();

        // Регистрируем в локальном registry
        let key = (self.uuid, Req::stable_type_hash(), Res::stable_type_hash());
        get_local_registry().insert(key, ListenerHandle { accept_tx });

        // Возвращаем listener
        Listener {
            accept_rx,
            _phantom: PhantomData,
        }
    }
}

#[derive(Builder)]
pub struct TransportClient<Req: Reflectionable, Res: Reflectionable> {
    uuid: Uuid,
    #[builder(default = Duration::from_secs(5))]
    timeout: Duration,
    #[builder(default = 3)]
    retry_tries: i32,
    #[builder(default)]
    error_strategy: ErrorStrategy,
    #[builder(default)]
    guarantees: Guarantees,
    #[builder(default)]
    _phantom: PhantomData<(Req, Res)>,
}

impl<Req: 'static + Reflectionable, Res: 'static + Reflectionable> TransportClient<Req, Res> {
    pub async fn create(self) -> Result<Connection<Req, Res>, ConnectError> {
        // Ищем в локальном registry
        let key = (self.uuid, Req::stable_type_hash(), Res::stable_type_hash());
        let handle = get_local_registry()
            .get(&key)
            .ok_or(ConnectError::ServiceNotFound)?;

        // Создаём пару inproc endpoints
        let (client_ep, server_ep) = InprocEndpoint::<Req, Res>::new_pair();

        // Отправляем server endpoint в listener
        let server_arc: Arc<dyn DynEndpoint> = Arc::new(server_ep);
        handle
            .accept_tx
            .send(crate::IncomingEndpoint::Inproc(server_arc))
            .await
            .map_err(|_| ConnectError::ListenerClosed)?;

        // Возвращаем connection с client endpoint
        let client_arc: Arc<dyn DynEndpoint> = Arc::new(client_ep);
        Ok(Connection {
            endpoint: client_arc,
            _phantom: PhantomData,
        })
    }
}

#[derive(Debug, Error)]
pub enum ConnectError {
    #[error("Service not found")]
    ServiceNotFound,
    #[error("Listener closed")]
    ListenerClosed,
}
