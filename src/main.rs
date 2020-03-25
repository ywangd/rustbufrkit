mod deser;
mod bufr;

#[allow(unused_variables)]
#[allow(unused_imports)]
use clap::{App, Arg, SubCommand};
use std::io;
use std::io::{BufRead};
use crate::deser::{BinaryDeser, Deserializer};

fn main() {
    let matches = App::new("RustBufrKit")
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
        .get_matches();

    let mut input_file = "";
    if let Some(matches) = matches.subcommand_matches("decode") {
        input_file = matches.value_of("INPUT").unwrap();
    }

    println!("{}", input_file);



    let mut bs = BinaryDeser {};
    if input_file == "-" {
        bs.deserialize(io::stdin().lock());
    }


}
