use std::io::BufReader;
use crate::decoder::find_string;

#[test]
fn test_find_string() {
    assert_eq!(find_string("BUFR", &mut "xxBUFRyyy7777zzz".as_bytes()).unwrap(), 2);
    assert_eq!(find_string("7777", &mut "xxBUFRyyy7777zzz".as_bytes()).unwrap(), 9);
}