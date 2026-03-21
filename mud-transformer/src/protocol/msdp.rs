use std::collections::HashMap;
use std::io::Write;
use std::iter::FusedIterator;
use std::{fmt, io};

use bytes::{Buf, Bytes};

/// MUD Server Data Protocol
///
/// https://tintin.mudhalla.net/protocols/msdp/
pub const OPT: u8 = 69;

pub const VAR: u8 = 1;
pub const VAL: u8 = 2;

pub const TABLE_OPEN: u8 = 3;
pub const TABLE_CLOSE: u8 = 4;

pub const ARRAY_OPEN: u8 = 5;
pub const ARRAY_CLOSE: u8 = 6;

pub fn parse(mut bytes: Bytes) -> Option<(Bytes, Data)> {
    take_var_and_value(&mut bytes)
}

#[derive(Clone, PartialEq, Eq)]
pub enum Value {
    String(Bytes),
    Array(Vec<Value>),
    Table(HashMap<Bytes, Value>),
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::String(bytes) => String::from_utf8_lossy(bytes).fmt(f),
            Self::Array(array) => array.fmt(f),
            Self::Table(table) => table.fmt(f),
        }
    }
}

impl Value {
    pub fn write_to<W: Write>(&self, mut writer: W) -> io::Result<()> {
        match self {
            Self::String(bytes) => writer.write_all(bytes),
            Self::Array(array) => {
                for val in array {
                    writer.write_all(&[VAL])?;
                    val.write_to(&mut writer)?;
                }
                Ok(())
            }
            Self::Table(table) => {
                for (var, val) in table {
                    writer.write_all(&[VAR])?;
                    writer.write_all(var)?;
                    writer.write_all(&[VAL])?;
                    val.write_to(&mut writer)?;
                }
                Ok(())
            }
        }
    }
}

impl From<Data> for Value {
    fn from(value: Data) -> Self {
        match value {
            Data::String(bytes) => Self::String(bytes),
            Data::Array(array) => Self::Array(array.into_values()),
            Data::Table(table) => Self::Table(table.into_values()),
        }
    }
}

impl<S: AsRef<[u8]>> From<S> for Value {
    fn from(value: S) -> Self {
        Self::String(Bytes::copy_from_slice(value.as_ref()))
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum Data {
    String(Bytes),
    Array(Array),
    Table(Table),
}

impl Data {
    pub fn into_value(self) -> Value {
        Value::from(self)
    }

    pub fn write_to<W: Write>(&self, mut writer: W) -> io::Result<()> {
        match self {
            Self::String(bytes) => writer.write_all(bytes),
            Self::Array(array) => {
                writer.write_all(&[ARRAY_OPEN])?;
                writer.write_all(&array.0)?;
                writer.write_all(&[ARRAY_CLOSE])
            }
            Self::Table(table) => {
                writer.write_all(&[TABLE_OPEN])?;
                writer.write_all(&table.0)?;
                writer.write_all(&[TABLE_CLOSE])
            }
        }
    }
}

impl fmt::Debug for Data {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::String(bytes) => String::from_utf8_lossy(bytes).fmt(f),
            Self::Array(array) => array.fmt(f),
            Self::Table(table) => table.fmt(f),
        }
    }
}

impl From<Bytes> for Data {
    fn from(mut bytes: Bytes) -> Self {
        match &*bytes {
            [ARRAY_OPEN, .., ARRAY_CLOSE] => {
                bytes.advance(1);
                bytes.truncate(bytes.len() - 1);
                Self::Array(Array(bytes))
            }
            [TABLE_OPEN, .., TABLE_CLOSE] => {
                bytes.advance(1);
                bytes.truncate(bytes.len() - 1);
                Self::Table(Table(bytes))
            }
            _ => Self::String(bytes),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Array(Bytes);

impl Array {
    pub fn into_values(self) -> Vec<Value> {
        self.map(Value::from).collect()
    }
}

impl fmt::Debug for Array {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl Iterator for Array {
    type Item = Data;

    fn next(&mut self) -> Option<Self::Item> {
        take_value(&mut self.0)
    }
}

impl FusedIterator for Array {}

#[derive(Clone, PartialEq, Eq)]
pub struct Table(Bytes);

impl Table {
    pub fn into_values(self) -> HashMap<Bytes, Value> {
        self.map(|(k, v)| (k, Value::from(v))).collect()
    }
}

impl fmt::Debug for Table {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_map().entries(self.clone()).finish()
    }
}

impl Iterator for Table {
    type Item = (Bytes, Data);

    fn next(&mut self) -> Option<Self::Item> {
        take_var_and_value(&mut self.0)
    }
}

impl FusedIterator for Table {}

fn take_value(bytes: &mut Bytes) -> Option<Data> {
    if !matches!(&**bytes, [VAL, ..]) {
        return None;
    }
    bytes.advance(1);
    let mut stack = 0i32;
    for (i, c) in bytes.iter().enumerate() {
        match *c {
            TABLE_OPEN | ARRAY_OPEN => stack += 1,
            TABLE_CLOSE | ARRAY_CLOSE => stack -= 1,
            VAR | VAL if stack == 0 => return Some(bytes.split_to(i).into()),
            _ => (),
        }
    }
    Some(bytes.split_off(0).into())
}

fn take_var_and_value(bytes: &mut Bytes) -> Option<(Bytes, Data)> {
    if !matches!(&**bytes, [VAR, ..]) {
        return None;
    }
    bytes.advance(1);
    let pos = bytes.iter().position(|&c| c == VAL)?;
    let var = bytes.split_to(pos);
    Some((var, take_value(bytes)?))
}
