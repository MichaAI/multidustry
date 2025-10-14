use binrw::{BinResult, binrw};
use cesu8::{from_java_cesu8, to_java_cesu8};

#[binrw]
#[brw(big)]
#[derive(Debug, Clone)]
pub struct NetworkIOString {
    len: u8,
    #[br(parse_with = parce_cesu8, args(len))]
    #[bw(write_with = write_cesu8)]
    pub value: String,
}

#[binrw::parser(reader)]
fn parce_cesu8(len: u8) -> BinResult<String> {
    let mut buf = vec![0u8; len as usize];
    reader.read(&mut buf);
    return Ok(from_java_cesu8(&buf)
        .map_err(|e| binrw::Error::Custom {
            pos: 0,
            err: Box::new(e),
        })?
        .into_owned());
}

#[binrw::writer(writer)]
fn write_cesu8(text: &String) -> BinResult<()> {
    let cesu8_bytes = to_java_cesu8(&text).into_owned();
    writer.write(&cesu8_bytes);
    Ok(())
}

impl NetworkIOString {
    fn new(text: String) -> Self {
        let text: String = text.chars().take(u8::MAX as usize).collect();
        NetworkIOString {
            len: text.len() as u8,
            value: text,
        }
    }
}

impl From<String> for NetworkIOString {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for NetworkIOString {
    fn from(value: &str) -> Self {
        Self::new(value.into())
    }
}
