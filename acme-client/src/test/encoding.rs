use crate::encoding::{b64_decode, b64_encode};
    
#[test]
fn test_b64_encoding() {
    assert_eq!(b64_encode(b""),"");
    assert_eq!(b64_encode(b"f"),"Zg");
    assert_eq!(b64_encode(b"fo"),"Zm8");
    assert_eq!(b64_encode(b"foo"),"Zm9v");
}

#[test]
fn test_b64_decoding() {
    let mut res = b64_decode("Zg").unwrap();
    assert_eq!(String::from_utf8(res).unwrap(),"f");
    res = b64_decode("Zm8").unwrap();
    assert_eq!(String::from_utf8(res).unwrap(),"fo");
    res = b64_decode("Zm9v").unwrap();
    assert_eq!(String::from_utf8(res).unwrap(),"foo");
}