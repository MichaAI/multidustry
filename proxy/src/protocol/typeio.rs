use std::{borrow::Cow, result};

use bytes::{Buf, BufMut, Bytes, BytesMut};
use cesu8::{Cesu8DecodingError, from_java_cesu8, to_java_cesu8};
use thiserror::Error;

pub struct TypeIOString {}

#[derive(Debug, Error)]
pub enum TypeIOStringDecodeError {
    #[error("Can't decode cesu8 string")]
    BadCesu8(#[from] Cesu8DecodingError),
}

impl TypeIOString {
    pub fn encode(string: &str, buf: &mut BytesMut) -> Result<(), TypeIOStringDecodeError> {
        if string.len() == 0 {
            buf.put_u8(0);
            return Ok(());
        }
        buf.put_u8(1);
        let cesu8_bytes = to_java_cesu8(string);
        buf.put_u16(cesu8_bytes.len() as u16);
        buf.put(&cesu8_bytes[..]);
        Ok(())
    }

    pub fn decode(buf: &mut Bytes) -> Result<Option<String>, TypeIOStringDecodeError> {
        let exists = buf.get_u8();
        if exists == 0 {
            return Ok(None);
        }
        let len = buf.get_u16();
        let slice = buf.split_to(len as usize);
        let res = from_java_cesu8(&slice[..])?;

        return Ok(Some(res.into_owned()));
    }
}
