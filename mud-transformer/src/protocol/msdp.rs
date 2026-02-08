use std::collections::HashMap;
use std::fmt;

use bytes::{Buf, Bytes};

/// MUD Server Data Protocol
///
/// https://tintin.mudhalla.net/protocols/msdp/
pub const CODE: u8 = 69;

pub const VAR: u8 = 1;
pub const VAL: u8 = 2;

pub const TABLE_OPEN: u8 = 3;
pub const TABLE_CLOSE: u8 = 4;

pub const ARRAY_OPEN: u8 = 5;
pub const ARRAY_CLOSE: u8 = 6;

#[derive(Clone, PartialEq, Eq)]
pub enum MsdpValue {
    String(Bytes),
    Array(Vec<MsdpValue>),
    Table(HashMap<Vec<u8>, MsdpValue>),
}

impl Default for MsdpValue {
    fn default() -> Self {
        Self::String(Bytes::new())
    }
}

impl fmt::Debug for MsdpValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::String(s) => String::from_utf8_lossy(s).fmt(f),
            Self::Array(a) => f.debug_list().entries(a).finish(),
            Self::Table(t) => {
                let entries = t
                    .iter()
                    .map(|(name, value)| (String::from_utf8_lossy(name), value));
                f.debug_map().entries(entries).finish()
            }
        }
    }
}

fn consume(bytes: &mut Bytes, c: u8) -> bool {
    if bytes.first() == Some(&c) {
        bytes.advance(1);
        return true;
    }
    false
}

impl MsdpValue {
    pub(crate) fn parse(data: &[u8]) -> Option<(Bytes, Self)> {
        let start = data.iter().position(|&c| c == VAR)? + 1;
        let mut data = Bytes::copy_from_slice(data.get(start..)?);
        let name = Self::take_string(&mut data);
        if !consume(&mut data, VAL) {
            return None;
        }
        Some((name, Self::take_value(&mut data)))
    }

    fn take_string(data: &mut Bytes) -> Bytes {
        let i = data
            .iter()
            .position(|&c| c <= ARRAY_CLOSE)
            .unwrap_or(data.len());
        data.split_to(i)
    }

    fn take_value(data: &mut Bytes) -> Self {
        match data.first() {
            Some(&ARRAY_OPEN) => Self::take_array(data),
            Some(&TABLE_OPEN) => Self::take_table(data),
            _ => Self::String(Self::take_string(data)),
        }
    }

    fn take_array(data: &mut Bytes) -> Self {
        data.advance(1);
        let mut array = Vec::new();
        while consume(data, VAL) {
            let value = Self::take_value(data);
            array.push(value);
        }
        consume(data, ARRAY_CLOSE);
        Self::Array(array)
    }

    fn take_table(data: &mut Bytes) -> Self {
        data.advance(1);
        let mut map = HashMap::new();
        while consume(data, VAR) {
            let name = Self::take_string(data);
            if !consume(data, VAL) {
                break;
            }
            let value = Self::take_value(data);
            map.insert(name.to_vec(), value);
        }
        consume(data, TABLE_CLOSE);
        Self::Table(map)
    }
}

impl<T> From<T> for MsdpValue
where
    T: AsRef<[u8]>,
{
    fn from(value: T) -> Self {
        Self::String(Bytes::copy_from_slice(value.as_ref()))
    }
}
