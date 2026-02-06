#![cfg(feature = "serde")]

use mud_transformer::term;
use mxp::RgbColor;
use serde::de::DeserializeOwned;
use serde::ser::Serialize;

#[track_caller]
pub fn roundtrip_bytes<T>(value: &T) -> T
where
    T: Serialize + DeserializeOwned,
{
    let serialized = postcard::to_stdvec(value).expect("error serializing value");
    match postcard::from_bytes(&serialized) {
        Ok(deserialized) => deserialized,
        Err(e) => panic!(
            "error deserializing {}: {e}",
            serde_json::to_value(value).unwrap()
        ),
    }
}

#[track_caller]
fn roundtrip_json<T>(value: &T) -> T
where
    T: Serialize + DeserializeOwned,
{
    let serialized = serde_json::to_value(value).expect("error serializing value");
    match serde_json::from_value(serialized) {
        Ok(deserialized) => deserialized,
        Err(e) => panic!(
            "error deserializing {}: {e}",
            serde_json::to_value(value).unwrap()
        ),
    }
}

#[test]
fn rgbcolor_serde_bytes() {
    let color = RgbColor::hex(0x123456);
    let roundtrip = roundtrip_bytes(&color);
    assert_eq!(roundtrip, color);
}

#[test]
fn rgbcolor_serde_json() {
    let color = RgbColor::hex(0x123456);
    let roundtrip = roundtrip_json(&color);
    assert_eq!(roundtrip, color);
}

#[test]
fn term_mode_serde_bytes() {
    let modes = vec![
        term::Mode::Standard(0),
        term::Mode::Standard(u16::MAX),
        term::Mode::Private(0),
        term::Mode::Standard(u16::MAX),
    ];
    let roundtrip = roundtrip_bytes(&modes);
    assert_eq!(roundtrip, modes);
}

#[test]
fn term_mode_serde_json() {
    let modes = vec![
        term::Mode::Standard(0),
        term::Mode::Standard(u16::MAX),
        term::Mode::Private(0),
        term::Mode::Standard(u16::MAX),
    ];
    let roundtrip = roundtrip_json(&modes);
    assert_eq!(roundtrip, modes);
}
