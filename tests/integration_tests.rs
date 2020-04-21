use std::io::BufReader;
use std::fs::File;

use rustbufrkit::decoder::decode_binary;
use rustbufrkit::table::table::TableGroupManager;

#[test]
fn test_decode() {
    let table_group_manager = TableGroupManager::new();
    let mut r = BufReader::new(
        File::open("tests/data/contrived.bufr").unwrap());
    let bufr_message = decode_binary(&table_group_manager, &mut r).unwrap();
    println!("{:?}", bufr_message);
}