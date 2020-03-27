use super::*;
use crate::table::table::{TableGroup, TableGroupId, TableGroupManager};
use std::ptr;
use std::ops::Deref;

#[test]
fn test_table_group_manager() {
    let mut tgm = TableGroupManager::new();
    let tg_id = TableGroupId {
        base_dir: String::from("_definitions/tables"),
        master_table_number: 0,
        centre_number: 0,
        sub_centre_number: 0,
        version_number: 25,
    };
    let t1 = tgm.get_table_group(&tg_id).unwrap();
    let t2 = tgm.get_table_group(&tg_id).unwrap();
    assert_eq!(1, tgm.size());
    ptr::eq(t1.deref(), t2.deref());
}

#[test]
fn test_load_table_group() {
    let table_group = create_table_group();

    assert_eq!("id=_definitions/tables/0/0_0/25", format!("{}", table_group))
}

#[test]
fn test_lookup_descriptor() {
    let table_group = create_table_group();

    table_group.lookup(1001).unwrap();
    table_group.lookup(101000).unwrap();
    table_group.lookup(201011).unwrap();
    table_group.lookup(225255).unwrap();
    table_group.lookup(300002).unwrap();
}

#[test]
#[should_panic]
fn test_lookup_bad_descriptor() {
    let table_group = create_table_group();
    table_group.lookup(987654).unwrap();
}

#[test]
fn test_lookup_cnf() {
    let table_group = create_table_group();

    assert_eq!("REGION V", table_group.lookup_cnf(1003, 5).unwrap());
    assert_eq!("REGION V", table_group.lookup_cnf(1003, 5).unwrap());
}

#[test]
fn test_lookup_meta() {
    let table_group = create_table_group();

    assert_eq!(
        "Identification: Identifies origin and type of data",
        table_group.lookup_meta(1001).unwrap(),
    );

    assert_eq!(
        "Change data width: Add (YYY-128) bits to the data width given for each data element in Table B, other than CCITT IA5 (character) data, code or flag tables.",
        table_group.lookup_meta(201011).unwrap(),
    );

    assert_eq!(
        "Difference statistical values follow: The statistical values which follow relate to the data defined by the data present bit-map.",
        table_group.lookup_meta(225000).unwrap(),
    );

    assert_eq!(
        "Location and identification sequences",
        table_group.lookup_meta(301059).unwrap(),
    );
}

#[test]
fn test_data_category_of() {
    let table_group = create_table_group();

    assert_eq!(
        "Single level upper - air data (satellite)",
        table_group.data_category_of(5).unwrap(),
    );

    assert_eq!(
        "Reserved",
        table_group.data_category_of(100).unwrap(),
    );
}

fn create_table_group() -> TableGroup {
    TableGroup::load(&TableGroupId {
        base_dir: String::from("_definitions/tables"),
        master_table_number: 0,
        centre_number: 0,
        sub_centre_number: 0,
        version_number: 25,
    }).unwrap()
}