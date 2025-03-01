use crate::encoding::{decode, encode};
    
#[test]
fn test_encoding() {
    assert_eq!(encode(b""),    "");
    assert_eq!(encode(b"f"),   "Zg");
    assert_eq!(encode(b"fo"),  "Zm8");
    assert_eq!(encode(b"foo"), "Zm9v");
}

#[test]
fn test_decoding() {
    let mut res = decode("Zg").unwrap();
    assert_eq!(String::from_utf8(res).unwrap(),"f");
    res = decode("Zm8").unwrap();
    assert_eq!(String::from_utf8(res).unwrap(),"fo");
    res = decode("Zm9v").unwrap();
    assert_eq!(String::from_utf8(res).unwrap(),"foo");
}