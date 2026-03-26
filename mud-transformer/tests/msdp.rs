mod common;
use std::collections::HashMap;

use bytes::Bytes;
use common::transform;
use mud_transformer::escape::telnet;
use mud_transformer::opt::msdp;
use mud_transformer::output::{OutputFragment, TelnetFragment};

fn subnegotiate(bytes: &[u8]) -> Vec<u8> {
    let mut subnegotiation = Vec::with_capacity(bytes.len() + 5);
    subnegotiation.extend_from_slice(&[telnet::IAC, telnet::SB, 69]);
    subnegotiation.extend_from_slice(bytes);
    subnegotiation.extend_from_slice(&[telnet::IAC, telnet::SE]);
    subnegotiation
}

fn make_msdp_table(entries: &[(&str, &str)]) -> HashMap<Bytes, msdp::Value> {
    entries
        .iter()
        .map(|(name, value)| (name.as_bytes().to_vec().into(), (*value).into()))
        .collect()
}

#[track_caller]
fn expect_msdp(fragment: Option<OutputFragment>, name: &str, value: msdp::Value) {
    #[derive(Debug, PartialEq, Eq)]
    struct Msdp<'a> {
        name: &'a str,
        value: msdp::Value,
    }
    let expected = Msdp { name, value };
    let Some(OutputFragment::Telnet(TelnetFragment::Subnegotiation {
        code: msdp::OPT,
        data,
    })) = fragment
    else {
        panic!("expected TelnetFragment::Subnegotiation, got {fragment:?}");
    };
    let (name, value) = msdp::decode(data).unwrap();
    let name = String::from_utf8_lossy(&name);
    let value = value.into_value();
    let actual = Msdp { name: &name, value };
    assert_eq!(expected, actual);
}

#[test]
fn msdp_array() {
    let message: &[u8] =
        b"\x01REPORTABLE_VARIABLES\x02\x05\x02HEALTH\x02HEALTH_MAX\x02MANA\x02MANA_MAX\x06";
    let output = transform(subnegotiate(message)).output();

    let expected_array = vec![
        b"HEALTH".into(),
        b"HEALTH_MAX".into(),
        b"MANA".into(),
        b"MANA_MAX".into(),
    ];
    let mut iter = output.into_iter();
    expect_msdp(
        iter.next(),
        "REPORTABLE_VARIABLES",
        msdp::Value::Array(expected_array),
    );
    assert_eq!(iter.as_slice(), &[]);
}

#[test]
fn msdp_table() {
    let message: &[u8] = b"\x01ROOM\x02\x03\x01VNUM\x026008\x01NAME\x02The forest clearing\x01AREA\x02Haon Dor\x01TERRAIN\x02forest\x01EXITS\x02\x03\x01n\x026011\x01e\x026007\x04\x04";
    let output = transform(subnegotiate(message)).output();

    let exit_map = make_msdp_table(&[("n", "6011"), ("e", "6007")]);
    let exit_map = msdp::Value::Table(exit_map);
    let mut expected_map = make_msdp_table(&[
        ("VNUM", "6008"),
        ("NAME", "The forest clearing"),
        ("AREA", "Haon Dor"),
        ("TERRAIN", "forest"),
    ]);
    expected_map.insert(b"EXITS".to_vec().into(), exit_map);

    let mut iter = output.into_iter();
    expect_msdp(iter.next(), "ROOM", msdp::Value::Table(expected_map));

    assert_eq!(iter.as_slice(), &[]);
}
