use std::{any::Any, marker::PhantomData, sync::Arc};

use async_trait::async_trait;
use quinn::ReadExactError;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::Mutex,
};
use uuid::Uuid;

use crate::{DynEndpoint, RecvError, Reflectionable, SendError};

// Первый frame который отправляется по QUIC stream
#[derive(Serialize, Deserialize, Debug)]
struct ConnectionHeader {
    uuid: Uuid,
    req_type: String, // Req::stable_type_hash()
    res_type: String, // Res::stable_type_hash()
}

async fn send_frame(stream: &mut quinn::SendStream, data: &[u8]) -> Result<(), std::io::Error> {
    let len = data.len() as u32;
    stream.write_u32_le(len).await?;
    stream.write_all(data).await?;
    Ok(())
}

async fn recv_frame(stream: &mut quinn::RecvStream) -> Result<Vec<u8>, RecvFrameError> {
    let len = stream.read_u32_le().await?;

    let mut data = Vec::with_capacity(len as usize);
    stream.read_exact(&mut data).await?;

    Ok(data)
}

#[derive(Debug, Error)]
pub enum RecvFrameError {
    #[error("Io Error")]
    IOError(#[from] std::io::Error),
    #[error("Read exact error")]
    ReadExactError(#[from] ReadExactError),
}

pub struct QuicEndpoint<Req, Res> {
    send_stream: Arc<Mutex<quinn::SendStream>>,
    recv_stream: Arc<Mutex<quinn::RecvStream>>,
    _phantom: PhantomData<(Req, Res)>,
}

impl<Req: Reflectionable, Res: Reflectionable> QuicEndpoint<Req, Res> {
    pub fn new(send: quinn::SendStream, recv: quinn::RecvStream) -> Self {
        Self {
            send_stream: Arc::new(Mutex::new(send)),
            recv_stream: Arc::new(Mutex::new(recv)),
            _phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<Req: 'static + Reflectionable, Res: 'static + Reflectionable> DynEndpoint
    for QuicEndpoint<Req, Res>
{
    async fn send_boxed(&self, msg: Box<dyn Any + Send>) -> Result<(), SendError> {
        // Downcast в Req
        let msg = msg.downcast::<Req>().map_err(|_| SendError::TypeMismatch)?;

        // Сериализуем в CBOR
        let mut buf = Vec::new();
        ciborium::into_writer(&*msg, &mut buf).map_err(|_| SendError::SerializationError)?;

        // Отправляем фрейм
        let mut stream = self.send_stream.lock().await;
        send_frame(&mut *stream, &buf).await?;

        Ok(())
    }

    async fn recv_boxed(&self) -> Result<Box<dyn Any + Send>, RecvError> {
        // Получаем фрейм
        let mut stream = self.recv_stream.lock().await;
        let data = recv_frame(&mut *stream).await?;

        // Десериализуем из CBOR
        let msg: Res =
            ciborium::from_reader(&data[..]).map_err(|_| RecvError::DeserializationError)?;

        Ok(Box::new(msg) as Box<dyn Any + Send>)
    }
}
