#![allow(unused)]

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate lazy_static;

mod table;
mod bufr;
pub mod decoder;
mod cmd;

#[cfg(test)]
mod tests;

use std::io::{Error};
use std::fs::File;

use clap::{App, Arg, SubCommand, ArgMatches};
use crate::cmd::{Command, DecodeCommand, LookupCommand};
use bitreader::BitReaderError;
use std::num::ParseIntError;


#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct BufrKitError {
    message: String,
}

impl From<Error> for BufrKitError {
    fn from(e: Error) -> Self {
        BufrKitError {
            message: e.to_string()
        }
    }
}

impl From<BitReaderError> for BufrKitError {
    fn from(e: BitReaderError) -> Self {
        BufrKitError {
            message: format!("{}", e)
        }
    }
}

impl From<ParseIntError> for BufrKitError {
    fn from(e: ParseIntError) -> Self {
        BufrKitError {
            message: format!("{}", e)
        }
    }
}

impl From<serde_json::Error> for BufrKitError {
    fn from(e: serde_json::Error) -> Self {
        BufrKitError {
            message: e.to_string()
        }
    }
}

pub fn new_app<'a, 'b>() -> App<'a, 'b> {
    App::new("RustBufrKit")
        .version("0.0.1")
        .author("ywangd@gmail.com")
        .about("Toolkit to work with WMO BUFR messages")
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .value_name("CONFIG_FILE")
            .help("Configuration file")
            .takes_value(true))
        .subcommand(SubCommand::with_name("decode")
            .about("Decode BUFR messages")
            .arg(Arg::with_name("INPUT")
                .help("Input file")
                .default_value("-")
                .required(false)
                .index(1)))
        .subcommand(SubCommand::with_name("encode")
            .about("Encode BUFR messages")
            .arg(Arg::with_name("INPUT")
                .help("Input file")
                .default_value("-")
                .required(false)
                .index(1)))
        .subcommand(SubCommand::with_name("lookup")
            .about("Lookup BUFR descriptors")
            .arg(Arg::with_name("IDS")
                .help("Comma separated list of descriptor IDs")
                .required(true)
                .index(1)))
}

pub fn run_app() -> Result<(), BufrKitError> {
    let matches = new_app().get_matches();
    match matches.subcommand() {
        ("decode", Some(sub_m)) => run_decoder(sub_m),
        ("encode", Some(sub_m)) => unimplemented!("encode"),
        ("lookup", Some(sub_m)) => run_lookup(sub_m),
        (s, _) => Err(BufrKitError {
            message: format!("Unknown command: [{}]", s)
        })
    }
}

fn run_decoder(matches: &ArgMatches) -> Result<(), BufrKitError> {
    let input_file = matches.value_of("INPUT").unwrap();
    let mut cmd = DecodeCommand::new(input_file);
    cmd.run()
}

fn run_lookup(matches: &ArgMatches) -> Result<(), BufrKitError> {
    let ids = matches.value_of("IDS").unwrap();
    let mut cmd = LookupCommand::new(ids);
    cmd.run()
}
