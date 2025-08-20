use std::io;

use shared::seq_bin::SeqBin;

fn main() -> io::Result<()> {
    let s = "Hello, World".to_string();
    let mut data: Vec<u8> = vec![];
    s.write_to(&mut data).unwrap();
    println!("{:?}", data);

    let s = String::read_from(&mut data.as_slice()).unwrap();
    println!("{:?}", s);

    Ok(())
}
