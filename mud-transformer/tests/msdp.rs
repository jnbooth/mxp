mod common;
use std::collections::HashMap;

use common::transform;
use mud_transformer::{MsdpValue, TelnetFragment};
use mxp::escape::telnet;

fn subnegotiate(bytes: &[u8]) -> Vec<u8> {
    let mut subnegotiation = Vec::with_capacity(bytes.len() + 5);
    subnegotiation.extend_from_slice(&[telnet::IAC, telnet::SB, 69]);
    subnegotiation.extend_from_slice(bytes);
    subnegotiation.extend_from_slice(&[telnet::IAC, telnet::SE]);
    subnegotiation
}

fn make_msdp_table(entries: &[(&str, &str)]) -> HashMap<Vec<u8>, MsdpValue> {
    entries
        .iter()
        .map(|(name, value)| (name.as_bytes().to_vec(), (*value).into()))
        .collect()
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
    let expected = &[
        TelnetFragment::Msdp {
            name: b"REPORTABLE_VARIABLES".as_slice().into(),
            value: MsdpValue::Array(expected_array),
        }
        .into(),
        TelnetFragment::Subnegotiation {
            code: 69,
            data: message.into(),
        }
        .into(),
    ];
    assert_eq!(output, expected);
}

#[test]
fn msdp_table() {
    let message: &[u8] = b"\x01ROOM\x02\x03\x01VNUM\x026008\x01NAME\x02The forest clearing\x01AREA\x02Haon Dor\x01TERRAIN\x02forest\x01EXITS\x02\x03\x01n\x026011\x01e\x026007\x06\x06";
    let output = transform(subnegotiate(message)).output();

    let exit_map = make_msdp_table(&[("n", "6011"), ("e", "6007")]);
    let exit_map = MsdpValue::Table(exit_map);
    let mut expected_map = make_msdp_table(&[
        ("VNUM", "6008"),
        ("NAME", "The forest clearing"),
        ("AREA", "Haon Dor"),
        ("TERRAIN", "forest"),
    ]);
    expected_map.insert(b"EXITS".to_vec(), exit_map);

    let expected = &[
        TelnetFragment::Msdp {
            name: b"ROOM".as_slice().into(),
            value: MsdpValue::Table(expected_map),
        }
        .into(),
        TelnetFragment::Subnegotiation {
            code: 69,
            data: message.into(),
        }
        .into(),
    ];
    assert_eq!(output, expected);
}
