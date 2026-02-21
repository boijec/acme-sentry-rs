use crate::crypto::SupportedKey;
use crate::jws::{JWSHeader, KeyFetcher, JWS};
use crate::keys::PrivateKey;
use openssl::pkey::{PKey, Public};
use serde_json::json;
use std::error::Error;

pub struct Fetcher {
    url: String,
}

impl KeyFetcher for Fetcher {
    fn fetch_key(&self, kid: String) -> Result<PKey<Public>, Box<dyn Error>> {
        panic!("Should not be called");
    }
}

#[test]
fn test_signing_jws() {
    let pkey = PrivateKey::from_supported_type(SupportedKey::EcP521).unwrap();
    let jws = JWS::with_header_and_payload(JWSHeader::with_alg(SupportedKey::EcP521.get_key_alg()), json!({
        "testPayload": "test",
    }));
    let t = jws.finalize(&pkey).unwrap();
    println!("{}", t);
}

#[test]
fn test_deserialize() {
    let string = "eyJhbGciOiJSUzI1NiIsImp3ayI6eyJhbGciOiJSUzI1NiIsImUiOiJBUUFCIiwia3R5IjoiUlNBIiwibiI6Im1fN2hmTlNaeGg3Z1paMkoxamVGb1hYaDVsT2NKY3NCM2pxNmpUTmNwTm5Pc1JJSlRsWmZaSkhia3NQbktnTHhHVmJxVUJNWWZrSnJuSjVQYzFrX0duV3JSSVBDRU1ILV9maXo1SlBtZ1pPWHV5RWRHU3V1MXViMjBHWWV5bW9KT2s5QTZoYlJnUGZqZWpSOGZBZ2J4MklFMEhyMHJmeE1kQm95bHNGbHlBbjRUa3RYQkRkdWdYaUlFaEtVN1k4VnB2eVB2VHpIMERMR2IzWjUyakxDX3dZS0VteVdsNDh3TE1Edi1nVlJKeXhZTWtOLWgweV9xZzNsdHJhNjVRaERMUkVvZEZueHd2Y1VIQk94OUQ1SVRVbU9SUUlEM0xSUGF2bExGOV9uWUQtUlQ5SHpIWmZtZUIxcERhNUxSd2MyckxwNGtSMXh2X05wckJTbG9lb2l0N1Jtby1hYzVKbTJYcGE4VHNsaXZDbEdiYjNFNzdLbDNMRTFGZWdMZ1RCNXZaUVVxYkRUZ1Q3V0MyWUJNanlQdWJhNHMyMGl4YlkzdjdZcW9TdnJhTFVTVVZudGpuOGdnd1BjTmxtdGpPX3NRRElUdzd6Mk9VZHJ5aWZGVG54V2tVcmNFRlI0VDNIWDRzVUNlclEwdGQyYzFPdkp4eC1KbGM2eXBJSUJ5UjhlWVNYMVZ4YWh0cE80b3B2SkpHYW5EQXNjY2FpOUM2UXJ4UklLU1YwMWF3eEZYYVVTM2pqanJxeEFWUUZ2blhZa01ZRWRfZUJTRThlU1FrazZOcXFBTEJHdnRSMjdyQmZtbmljdEw2dkszYVNlbnZmUS16eDR5ZTBHWTN1eWg0OFM2MlZtMmd4R1Z6ZFBqcnFSNHREaEswZ0p0bXBJSFh6M1RVYW9CbmRhRmpFIn0sIm5vbmNlIjoicGxhY2Vob2xkZXIiLCJ1cmwiOiIifQ.eyJ0ZXN0UGF5bG9hZCI6InRlc3QifQ.KNi9z8lal5w6hbFG2j557QC-U_yLEL_VR-lt4ZynP4HFmESSk2nahxcXrqJ9bzreN6knnyJ8cBERt4HUbci5z5T_lsFp5PBBsmRBoQeXOMKOMICKRYOuX7QTgwPt6eamAaI-Gjdq0-vdDNdTPMA1rxdQGWL6sbCNH8A-QLKaK4qFRrTBo8rEf6me1HV5brB6bCCFJnJQ1Ou2LQjD0JgKwguWtxeFdNAU-jOrClLV92vbtcbwJb4540AGdTtbtvOhce9PfMBtYerRe6dnrhpuI4TcYWpm8sfxWQdOi5Y9HkZ7VnF8Kp6VXXLvcSZhHNdlXqKgd8fIJLa0qr2h8oKK3gHOF_KJHVHC-LI-xdILIeJP7bTHExkwBiWiAfu3hduLJpQ_sSvWcbYevHmEFWLog1a2yy1g-TNddawlX67cw-dm_ZNDIFqoJPLjHbLDfuwMAXKSBOwXXrWgWZ7JxGs80mNoqeoe1mOfe1QNKM0cSlAqZoxVwi5sFYad6PnpS-swPiggeLCvY9JLCwIb9juMchSCO9zjMI3yxDRoR5bPcoa1q7lYTwGj7Q0qreNvPyUlqxuQ0mibIo5OU9aNQDY6B9rjN3CaYGpM_5-y6Mt2X38abSWPqFLTZjPp0fS_bfMSHz-IJdis2HUYhbV3wBytWCeamvuNUs3MMrwox5YBPHA";
    let jws = JWS::parse(&string.to_string(), Box::new(Fetcher { url: "http://localhost:8080".to_string() })).unwrap().unwrap();
    let payload = jws.get_payload();
    assert_eq!(payload, &json!({
        "testPayload": "test",
    }))
}