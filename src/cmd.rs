use std::io::{BufReader};
use crate::decoder::{decode_binary};
use crate::BufrKitError;
use std::io;
use std::fs::File;

pub trait Command {
    fn run(&mut self) -> Result<(), BufrKitError>;
}

pub struct DecodeCommand<'a> {
    ins_name: &'a str
}

impl<'a> DecodeCommand<'a> {
    pub fn new(ins_name: &'a str) -> Self {
        return DecodeCommand {
            ins_name,
        };
    }
}

impl<'a> Command for DecodeCommand<'a> {
    fn run(&mut self) -> Result<(), BufrKitError> {
        let bufr_message = if self.ins_name == "-" {
            decode_binary(&mut io::stdin().lock())?
        } else {
            let file = File::open(self.ins_name).unwrap();
            decode_binary(&mut BufReader::new(file))?
        };
        println!("{:?}", bufr_message);
        Ok(())
    }
}