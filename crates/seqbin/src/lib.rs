use std::io::{self, Read, Write};

#[derive(Debug)]
pub enum SeqBinError {
    Io(io::Error),
    UnexpectedEof,
}

impl From<io::Error> for SeqBinError {
    fn from(e: io::Error) -> Self {
        if e.kind() == io::ErrorKind::UnexpectedEof {
            SeqBinError::UnexpectedEof
        } else {
            SeqBinError::Io(e)
        }
    }
}

pub trait SeqBin: Sized {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), SeqBinError>;
    fn read_from<R: Read>(reader: &mut R) -> Result<Self, SeqBinError>;
}

macro_rules! impl_seqbin_int {
    ($t:ty) => {
        impl SeqBin for $t {
            fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), SeqBinError> {
                writer
                    .write_all(&self.to_be_bytes())
                    .map_err(SeqBinError::from)
            }

            fn read_from<R: Read>(reader: &mut R) -> Result<Self, SeqBinError> {
                let mut buf = [0u8; std::mem::size_of::<$t>()];
                reader.read_exact(&mut buf).map_err(SeqBinError::from)?;
                Ok(<$t>::from_be_bytes(buf))
            }
        }
    };
}

impl_seqbin_int!(u8);
impl_seqbin_int!(i8);
impl_seqbin_int!(u16);
impl_seqbin_int!(i16);
impl_seqbin_int!(u32);
impl_seqbin_int!(i32);
impl_seqbin_int!(u64);
impl_seqbin_int!(i64);
impl_seqbin_int!(u128);
impl_seqbin_int!(i128);
impl_seqbin_int!(f32);
impl_seqbin_int!(f64);

impl<T: SeqBin> SeqBin for Vec<T> {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), SeqBinError> {
        let len = self.len() as u32;
        len.write_to(writer)?;
        for item in self {
            item.write_to(writer)?;
        }
        Ok(())
    }

    fn read_from<R: Read>(reader: &mut R) -> Result<Self, SeqBinError> {
        let len = u32::read_from(reader)? as usize;
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(T::read_from(reader)?);
        }
        Ok(vec)
    }
}

impl SeqBin for String {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), SeqBinError> {
        let bytes = self.as_bytes();
        let len = bytes.len() as u32;
        len.write_to(writer)?;
        writer.write_all(bytes)?;
        Ok(())
    }

    fn read_from<R: Read>(reader: &mut R) -> Result<Self, SeqBinError> {
        let len = u32::read_from(reader)? as usize;
        let mut buf = vec![0u8; len];
        reader.read_exact(&mut buf)?;
        Ok(String::from_utf8(buf).map_err(|_| SeqBinError::UnexpectedEof)?)
    }
}
