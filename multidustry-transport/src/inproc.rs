use std::any::Any;

use async_trait::async_trait;

use crate::{DynEndpoint, Needed, RecvError, SendError};

pub struct InprocEndpoint<Req: 'static + Needed, Res: 'static + Needed> {
    server_tx: kanal::AsyncSender<Req>,
    server_rx: kanal::AsyncReceiver<Res>,
    client_tx: kanal::AsyncSender<Req>,
    client_rx: kanal::AsyncReceiver<Res>,
}

impl<Req: Needed, Res: Needed> InprocEndpoint<Req, Res> {}

#[async_trait]
impl<Req: 'static + Needed, Res: 'static + Needed> DynEndpoint for InprocEndpoint<Req, Res> {
    fn req_type(&self) -> &'static str {
        Req::stable_type_hash()
    }

    fn res_type(&self) -> &'static str {
        Res::stable_type_hash()
    }

    async fn send_boxed(&self, msg: Box<dyn Any + Send>) -> Result<(), SendError> {
        let msg = msg.downcast::<Req>().map_err(|_| SendError::TypeMismatch)?;
        self.tx.send(*msg).await.map_err(SendError::from)
    }

    async fn recv_boxed(&self) -> Result<Box<dyn Any + Send>, RecvError> {
        let val = self.rx.recv().await.map_err(|_| RecvError::ChannelClosed)?;
        Ok(Box::new(val) as Box<dyn Any + Send>)
    }
}
