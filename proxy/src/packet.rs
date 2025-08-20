use std::io::{Read, Write};

use bincode::{Decode, Encode};
use lz4_flex::{compress_prepend_size, decompress_size_prepended};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PacketError {
    #[error("IO error")]
    Io(#[from] std::io::Error),
    #[error("Encode error")]
    Encode(#[from] bincode::error::EncodeError),
    #[error("Decode error")]
    Decode(#[from] bincode::error::DecodeError),
    #[error("Decompression error")]
    Decompress,
    #[error("Packet ID mismatch")]
    IdMismatch,
}

pub trait Packet: Encode + Decode<()> {
    fn id(&self) -> u8;
}

pub fn write_packet<W: Write, P: Packet>(writer: &mut W, packet: &P) -> Result<(), PacketError> {
    let config = bincode::config::standard().with_big_endian();
    let buf = bincode::encode_to_vec(packet, config)?;

    if buf.len() >= 36 {
        let compressed = compress_prepend_size(&buf);
        writer.write_all(&[packet.id()])?;
        writer.write_all(&(compressed.len() as u16).to_be_bytes())?;
        writer.write_all(&[1u8])?;
        writer.write_all(&compressed)?;
    } else {
        writer.write_all(&[packet.id()])?;
        writer.write_all(&(buf.len() as u16).to_be_bytes())?;
        writer.write_all(&[0u8])?;
        writer.write_all(&buf)?;
    }

    todo!()
}

pub fn read_packet<R: Read, P: Packet>(reader: &mut R) -> Result<P, PacketError> {
    let config = bincode::config::standard().with_big_endian();

    let mut id_buf = [0u8; 1];
    reader.read_exact(&mut id_buf)?;
    let id = id_buf[0];

    let mut len_buf = [0u8; 2];
    reader.read_exact(&mut len_buf)?;
    let length = u16::from_be_bytes(len_buf) as usize;

    let mut comp_buf = [0u8; 1];
    reader.read_exact(&mut comp_buf)?;
    let compression = comp_buf[0];

    let mut buf = vec![0u8; length];
    reader.read_exact(&mut buf)?;

    let data = if compression == 1 {
        decompress_size_prepended(&buf).map_err(|_| PacketError::Decompress)?
    } else {
        buf
    };

    let (packet, _read_bytes) =
        bincode::decode_from_slice::<P, _>(&data, config).map_err(PacketError::Decode)?;

    if packet.id() != id {
        return Err(PacketError::IdMismatch);
    }

    Ok(packet)
}
