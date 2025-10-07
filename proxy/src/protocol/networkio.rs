use bytes::{Buf, BufMut, Bytes, BytesMut};
use cesu8::{Cesu8DecodingError, from_java_cesu8, to_java_cesu8};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NetworkIOStringError {
    #[error("Can't decode cesu8 string")]
    BadCesu8(#[from] Cesu8DecodingError),
}

pub struct NetworkIOString {}

impl NetworkIOString {
    pub fn encode(string: &str, buf: &mut BytesMut) -> Result<(), NetworkIOStringError> {
        let cesu_str = to_java_cesu8(string);
        buf.put_u8(cesu_str.len() as u8); // Len
        buf.put_slice(&cesu_str); // Data
        Ok(())
    }

    pub fn decode(buf: &mut Bytes) -> Result<String, NetworkIOStringError> {
        let len = buf.get_u8();
        let slice = buf.split_to(len as usize);
        let res = from_java_cesu8(&slice)?;
        Ok(res.into_owned())
    }
}
