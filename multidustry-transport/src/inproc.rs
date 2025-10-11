use std::any::Any;

use async_trait::async_trait;
use kanal::{AsyncReceiver, AsyncSender};

use crate::{DynEndpoint, RecvError, Reflectionable, SendError};

pub struct InprocEndpoint<Req, Res> {
    tx: AsyncSender<Req>,
    rx: AsyncReceiver<Res>,
}

impl<Req: Reflectionable, Res: Reflectionable> InprocEndpoint<Req, Res> {
    pub fn new_pair() -> (InprocEndpoint<Req, Res>, InprocEndpoint<Res, Req>) {
        let (req_tx, req_rx) = kanal::unbounded_async();
        let (res_tx, res_rx) = kanal::unbounded_async();

        let client = InprocEndpoint {
            tx: req_tx, // client отправляет запросы
            rx: res_rx, // client получает ответы
        };

        let server = InprocEndpoint {
            tx: res_tx, // server отправляет ответы
            rx: req_rx, // server получает запросы
        };

        (client, server)
    }

    // Send message
    pub async fn send(&self, msg: Req) -> Result<(), SendError> {
        self.tx
            .send(msg)
            .await
            .map_err(|_| SendError::ChannelClosed)
    }

    // Recieve message
    pub async fn recv(&self) -> Result<Res, RecvError> {
        self.rx.recv().await.map_err(|_| RecvError::ChannelClosed)
    }
}

#[async_trait]
impl<Req: 'static + Reflectionable, Res: 'static + Reflectionable> DynEndpoint
    for InprocEndpoint<Req, Res>
{
    async fn send_boxed(&self, msg: Box<dyn Any + Send>) -> Result<(), SendError> {
        // Пытаемся downcast Box<dyn Any> обратно в Req
        let msg = msg.downcast::<Req>().map_err(|_| SendError::TypeMismatch)?;

        // Отправляем через наш канал
        self.tx
            .send(*msg)
            .await
            .map_err(|_| SendError::ChannelClosed)
    }

    async fn recv_boxed(&self) -> Result<Box<dyn Any + Send>, RecvError> {
        // Получаем из канала
        let val = self.rx.recv().await.map_err(|_| RecvError::ChannelClosed)?;

        // Упаковываем в Box<dyn Any>
        Ok(Box::new(val) as Box<dyn Any + Send>)
    }
}
