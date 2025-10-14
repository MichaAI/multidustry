use binrw::{BinRead, BinResult, BinWrite, binrw};
use cesu8::to_java_cesu8;

#[binrw]
#[brw(big)]
#[derive(Debug, Clone)]
pub struct TypeIOString {
    pub exists: u8,

    #[br(if(exists != 0))]
    #[bw(if(*exists != 0))]
    #[br(parse_with = parce_cesu8_optional, args(exists))]
    #[bw(write_with = write_cesu8, args(exists))]
    pub value: Option<String>,
}

#[binrw::parser(reader)]
fn parce_cesu8_optional(exists: u8) -> BinResult<Option<String>> {
    if exists == 0 {
        return Ok(None);
    }
    let len = u16::read_be(reader)?.into();
    let mut buf = vec![0u8; len];
    let str_len = reader.read(&mut buf)?;
    if len != str_len {
        return Err(binrw::Error::Custom {
            pos: 0,
            err: Box::new(0),
        });
    }
    return Ok(Some(
        cesu8::from_java_cesu8(&buf)
            .map_err(|e| binrw::Error::Custom {
                pos: 0,
                err: Box::new(e),
            })?
            .into_owned(),
    ));
}

#[binrw::writer(writer)]
fn write_cesu8(text: &Option<String>, exists: &u8) -> BinResult<()> {
    if *exists == 0 {
        return Ok(());
    }

    match text {
        Some(text) => {
            if text.len() > u16::MAX.into() {
                let collect: String = text.chars().take(u16::MAX.into()).collect();
                (collect.len() as u16).write_be(writer);
                to_java_cesu8(&collect).write_args(writer, ());
                return Ok(());
            }

            (text.len() as u16).write_be(writer)?;
            to_java_cesu8(text).write_args(writer, ())
        }
        None => {
            0u16.write_be(writer)?;
            return Ok(());
        }
    }
}
