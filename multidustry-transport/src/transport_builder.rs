use std::{marker::PhantomData, sync::Arc};

use thiserror::Error;
use uuid::Uuid;

use crate::{
    Connection, DynEndpoint, Listener, ListenerHandle, Reflectionable, get_local_registry,
    inproc::InprocEndpoint,
};

pub struct TransportBuilder<Req: Reflectionable, Res: Reflectionable> {
    _phantom: PhantomData<(Req, Res)>,
}

impl<Req: 'static + Reflectionable, Res: 'static + Reflectionable> TransportBuilder<Req, Res> {
    pub fn server(uuid: Uuid) -> TransportServerBuilder<Req, Res> {
        TransportServerBuilder {
            uuid,
            _phantom: PhantomData,
        }
    }

    pub fn client(uuid: Uuid) -> TransportClientBuilder<Req, Res> {
        TransportClientBuilder {
            uuid,
            _phantom: PhantomData,
        }
    }
}

pub struct TransportServerBuilder<Req: Reflectionable, Res: Reflectionable> {
    uuid: Uuid,
    _phantom: PhantomData<(Req, Res)>,
}

impl<Req: 'static + Reflectionable, Res: 'static + Reflectionable>
    TransportServerBuilder<Req, Res>
{
    pub async fn build(self) -> Listener<Req, Res> {
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

pub struct TransportClientBuilder<Req: Reflectionable, Res: Reflectionable> {
    uuid: Uuid,
    _phantom: PhantomData<(Req, Res)>,
}

#[derive(Debug, Error)]
pub enum ConnectError {
    #[error("Service not found")]
    ServiceNotFound,
    #[error("Listener closed")]
    ListenerClosed,
}

impl<Req: 'static + Reflectionable, Res: 'static + Reflectionable>
    TransportClientBuilder<Req, Res>
{
    pub async fn build(self) -> Result<Connection<Req, Res>, ConnectError> {
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
            .send(server_arc)
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
