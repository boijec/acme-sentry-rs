use crate::encoding::{decode_b64, encode_b64};
    
#[test]
fn test_b64_encoding() {
    assert_eq!(encode_b64(b""),"");
    assert_eq!(encode_b64(b"f"),"Zg");
    assert_eq!(encode_b64(b"fo"),"Zm8");
    assert_eq!(encode_b64(b"foo"),"Zm9v");
}

#[test]
fn test_b64_decoding() {
    let mut res = decode_b64("Zg").unwrap();
    assert_eq!(String::from_utf8(res).unwrap(),"f");
    res = decode_b64("Zm8").unwrap();
    assert_eq!(String::from_utf8(res).unwrap(),"fo");
    res = decode_b64("Zm9v").unwrap();
    assert_eq!(String::from_utf8(res).unwrap(),"foo");
}