use crate::encoding::{count_occurrences, decode_b64, encode_b64, remove_first, remove_last};
    
#[test]
fn test_b64_encoding() {
    assert_eq!(encode_b64(b"testing tornado"),"dGVzdGluZyB0b3JuYWRv");
}

#[test]
fn test_b64_decoding() {
    let res = decode_b64("dGVzdGluZyB0b3JuYWRv").unwrap();
    assert_eq!(String::from_utf8(res).unwrap(),"testing tornado");
}

#[test]
fn test_remove_last_character() {
    let str = "asdf.asdf.";
    assert_eq!(remove_last(str),"asdf.asdf");
}

#[test]
fn test_remove_first_character() {
    let str = ".asdf.asdf";
    assert_eq!(remove_first(str),"asdf.asdf");
}

#[test]
fn test_count_method() {
    let str = "asdf.asdf.asdf";
    assert_eq!(count_occurrences(str, '.'),2);
}