use std::io::BufReader;
use std::fs::File;

use rustbufrkit::decoder::decode_binary;

#[test]
fn test_decode() {
    let mut r = BufReader::new(
        File::open("tests/data/contrived.bufr").unwrap());
    let bufr_message = decode_binary(&mut r).unwrap();
    println!("{:?}", bufr_message);
}