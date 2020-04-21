use std::io::BufReader;
use crate::decoder::decode_binary;
use crate::BufrKitError;
use std::io;
use std::fs::File;
use crate::table::template::{Template, PrintVisitor};
use crate::table::table::{TableGroup, TableGroupId, TableGroupManager};

pub trait Command {
    fn run(&mut self) -> Result<(), BufrKitError>;
}

pub struct DecodeCommand<'a> {
    ins_name: &'a str
}

impl<'a> DecodeCommand<'a> {
    pub fn new(ins_name: &'a str) -> Self {
        DecodeCommand {
            ins_name,
        }
    }
}

impl<'a> Command for DecodeCommand<'a> {
    fn run(&mut self) -> Result<(), BufrKitError> {
        let table_group_manager = TableGroupManager::new();
        let bufr_message = if self.ins_name == "-" {
            decode_binary(&table_group_manager, &mut io::stdin().lock())?
        } else {
            let file = File::open(self.ins_name).unwrap();
            decode_binary(&table_group_manager, &mut BufReader::new(file))?
        };
        println!("{:?}", bufr_message);
        Ok(())
    }
}

pub struct LookupCommand<'a> {
    ids: &'a str,
}

impl<'a> LookupCommand<'a> {
    pub fn new(ids: &'a str) -> Self {
        LookupCommand {
            ids,
        }
    }
}

impl<'a> Command for LookupCommand<'a> {
    fn run(&mut self) -> Result<(), BufrKitError> {
        let mut ids = Vec::new();
        for s in self.ids.split(",") {
            ids.push(s.parse::<isize>()?);
        }
        let table_group = TableGroupManager::new().get_table_group(&TableGroupId {
            base_dir: String::from("_definitions/tables"),
            master_table_number: 0,
            centre_number: 0,
            sub_centre_number: 0,
            version_number: 25,
        }).unwrap();
        let template = Template::new(&table_group, &ids)?;
        template.accept(&mut PrintVisitor::new());
        Ok(())
    }
}
