use std::io::BufRead;
use crate::bufr::BufrMessage;

pub trait Deserializer {
    fn deserialize<T: BufRead>(& mut self, r: T) -> BufrMessage;
}


pub struct BinaryDeser {}

impl Deserializer for BinaryDeser {
    fn deserialize<T: BufRead>(& mut self, mut r: T) -> BufrMessage {
        println!("deserialize");
        let mut buffer = String::new();

        let nbytes = r.read_line(&mut buffer)
            .expect("wrong");
        println!("{} {}", nbytes, buffer);


        return BufrMessage { version: "4" };

    }
}