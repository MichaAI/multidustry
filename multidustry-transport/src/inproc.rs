use std::any::Any;

use async_trait::async_trait;

use crate::{DynEndpoint, RecvError, Reflectionable, SendError};

pub struct InprocEndpoint<Req: 'static + Reflectionable, Res: 'static + Reflectionable> {
    tx: kanal::AsyncSender<Req>,
    rx: kanal::AsyncReceiver<Res>,
}

impl<Req: Reflectionable, Res: Reflectionable> InprocEndpoint<Req, Res> {}

#[async_trait]
impl<Req: 'static + Reflectionable, Res: 'static + Reflectionable> DynEndpoint
    for InprocEndpoint<Req, Res>
{
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
