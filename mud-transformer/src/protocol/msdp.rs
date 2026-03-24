use std::borrow::Borrow;
use std::collections::HashMap;
use std::io::Write;
use std::iter::{self, FusedIterator};
use std::{fmt, io};

use bytes::{Buf, Bytes};

use crate::escape::telnet;

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

pub fn decode(mut bytes: Bytes) -> Option<(Bytes, Data)> {
    Data::take_var_and_value(&mut bytes)
}

fn write_var<W: Write>(writer: &mut W, var: &[u8]) -> io::Result<()> {
    writer.write_all(&[VAR])?;
    writer.write_all(var)?;
    writer.write_all(&[VAL])
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
            Self::String(bytes) => bytes.fmt(f),
            Self::Array(array) => array.fmt(f),
            Self::Table(table) => table.fmt(f),
        }
    }
}

impl Value {
    pub fn write_to<W: Write>(&self, mut writer: W) -> io::Result<()> {
        self.write_to_mut(&mut writer)
    }

    // Prevents trait recursion.
    fn write_to_mut<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        match self {
            Self::String(bytes) => writer.write_all(bytes),
            Self::Array(array) => {
                for val in array {
                    writer.write_all(&[VAL])?;
                    val.write_to_mut(writer)?;
                }
                Ok(())
            }
            Self::Table(table) => {
                for (var, val) in table {
                    write_var(writer, var)?;
                    val.write_to_mut(writer)?;
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
        Some((var, Data::take_value(bytes)?))
    }
}

impl fmt::Debug for Data {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::String(bytes) => bytes.fmt(f),
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
        Data::take_value(&mut self.0)
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
        Data::take_var_and_value(&mut self.0)
    }
}

impl FusedIterator for Table {}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Command<'a, I> {
    command: &'a str,
    params: I,
}

impl<'a, T> Command<'a, iter::Once<T>>
where
    T: AsRef<[u8]>,
{
    pub fn simple(command: &'a str, param: T) -> Self {
        Self {
            command,
            params: iter::once(param),
        }
    }

    pub fn list(param: T) -> Self {
        Self::simple("LIST", param)
    }

    pub fn reset(param: T) -> Self {
        Self::simple("RESET", param)
    }
}

impl<'a, I> Command<'a, I>
where
    I: IntoIterator,
    I::Item: AsRef<[u8]>,
{
    pub fn custom(command: &'a str, params: I) -> Self {
        Self { command, params }
    }

    pub fn report(params: I) -> Self {
        Self::custom("REPORT", params)
    }

    pub fn send(params: I) -> Self {
        Self::custom("SEND", params)
    }

    pub fn unreport(params: I) -> Self {
        Self::custom("UNREPORT", params)
    }

    pub fn write<W: Write>(self, mut writer: W) -> io::Result<()> {
        writer.write_all(&[telnet::IAC, telnet::SB, OPT, VAR])?;
        writer.write_all(self.command.as_bytes())?;
        for param in self.params {
            writer.write_all(&[VAL])?;
            writer.write_all(param.as_ref())?;
        }
        writer.write_all(&[telnet::IAC, telnet::SE])
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Encode<'a> {
    String(&'a [u8]),
    Array(&'a [Encode<'a>]),
    Table(&'a [(&'a [u8], Encode<'a>)]),
}

impl fmt::Debug for Encode<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::String(bytes) => bytes.fmt(f),
            Self::Array(array) => f.debug_list().entries(array).finish(),
            Self::Table(table) => f.debug_map().entries(table.iter().copied()).finish(),
        }
    }
}

impl Encode<'_> {
    pub fn write_to<W: Write>(&self, mut writer: W) -> io::Result<()> {
        self.write_to_mut(&mut writer)
    }

    // Prevents trait recursion.
    fn write_to_mut<W: Write>(&self, mut writer: &mut W) -> io::Result<()> {
        match *self {
            Self::String(bytes) => writer.write_all(bytes),
            Self::Array(array) => {
                for val in array {
                    writer.write_all(&[VAL])?;
                    val.write_to_mut(writer)?;
                }
                Ok(())
            }
            Self::Table(table) => {
                for (var, val) in table {
                    write_var(&mut writer, var)?;
                    val.write_to_mut(writer)?;
                }
                Ok(())
            }
        }
    }
}

impl<'a, T: AsRef<[u8]>> From<&'a T> for Encode<'a> {
    fn from(value: &'a T) -> Self {
        Encode::String(value.as_ref())
    }
}

pub trait Encodable {
    fn encode(&self, writer: &mut dyn Write) -> io::Result<()>;
}

pub struct EncodeStream<W: Write> {
    writer: W,
}

impl<W: Write> EncodeStream<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    pub fn into_inner(self) -> W {
        self.writer
    }

    pub fn start(&mut self) -> io::Result<()> {
        self.writer.write_all(&[telnet::IAC, telnet::SB, OPT])
    }

    pub fn put<K, V>(&mut self, var: K, val: &V) -> io::Result<()>
    where
        K: AsRef<[u8]>,
        V: Encodable,
    {
        write_var(&mut self.writer, var.as_ref())?;
        val.encode(&mut self.writer)
    }

    pub fn put_array<K>(&mut self, var: K, vals: &[&dyn Encodable]) -> io::Result<()>
    where
        K: AsRef<[u8]>,
    {
        self.put_array_entries(var, vals)
    }

    pub fn put_array_entries<K, I>(&mut self, var: K, vals: I) -> io::Result<()>
    where
        K: AsRef<[u8]>,
        I: IntoIterator,
        I::Item: Encodable,
    {
        write_var(&mut self.writer, var.as_ref())?;
        self.writer.write_all(&[ARRAY_OPEN])?;
        for val in vals {
            self.writer.write_all(&[VAL])?;
            val.encode(&mut self.writer)?;
        }
        self.writer.write_all(&[ARRAY_CLOSE])
    }

    pub fn put_table<K, K1>(&mut self, var: K, vals: &[(K1, &dyn Encodable)]) -> io::Result<()>
    where
        K: AsRef<[u8]>,
        K1: AsRef<[u8]>,
    {
        self.put_table_entries(var, vals)
    }

    pub fn put_table_entries<K, K1, I, V>(&mut self, var: K, vals: I) -> io::Result<()>
    where
        K: AsRef<[u8]>,
        I: IntoIterator,
        I::Item: Borrow<(K1, V)>,
        K1: AsRef<[u8]>,
        V: Encodable,
    {
        write_var(&mut self.writer, var.as_ref())?;
        self.writer.write_all(&[TABLE_OPEN])?;
        for entry in vals {
            let (var, val) = entry.borrow();
            self.put(var, &val)?;
        }
        self.writer.write_all(&[TABLE_CLOSE])
    }

    pub fn finish(&mut self) -> io::Result<()> {
        self.writer.write_all(&[telnet::IAC, telnet::SE])
    }
}

impl Encodable for &dyn Encodable {
    fn encode(&self, writer: &mut dyn Write) -> io::Result<()> {
        (**self).encode(writer)
    }
}

impl<T: Encodable> Encodable for &T {
    fn encode(&self, writer: &mut dyn Write) -> io::Result<()> {
        (*self).encode(writer)
    }
}

impl Encodable for [u8] {
    fn encode(&self, writer: &mut dyn Write) -> io::Result<()> {
        writer.write_all(self)
    }
}

impl Encodable for str {
    fn encode(&self, writer: &mut dyn Write) -> io::Result<()> {
        writer.write_all(self.as_bytes())
    }
}

impl Encodable for Bytes {
    fn encode(&self, writer: &mut dyn Write) -> io::Result<()> {
        writer.write_all(self)
    }
}

impl Encodable for Value {
    fn encode(&self, writer: &mut dyn Write) -> io::Result<()> {
        self.write_to(writer)
    }
}

impl Encodable for Data {
    fn encode(&self, writer: &mut dyn Write) -> io::Result<()> {
        self.write_to(writer)
    }
}

impl Encodable for Encode<'_> {
    fn encode(&self, writer: &mut dyn Write) -> io::Result<()> {
        self.write_to(writer)
    }
}

macro_rules! impl_encodable {
    ($t:ty) => {
        impl Encodable for $t {
            fn encode(&self, writer: &mut dyn Write) -> io::Result<()> {
                write!(writer, "{self}")
            }
        }
    };
}

impl_encodable!(u8);
impl_encodable!(u16);
impl_encodable!(u32);
impl_encodable!(u64);
impl_encodable!(usize);
impl_encodable!(i8);
impl_encodable!(i16);
impl_encodable!(i32);
impl_encodable!(i64);
impl_encodable!(isize);
impl_encodable!(f32);
impl_encodable!(f64);
