use crate::mxp;
use std::fmt::{self, Display, Formatter};
use std::io;

#[derive(Debug)]
pub enum ParseError {
    Mxp(mxp::ParseError),
    Io(io::Error),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Mxp(err) => err.fmt(f),
            Self::Io(err) => err.fmt(f),
        }
    }
}

impl From<mxp::ParseError> for ParseError {
    fn from(value: mxp::ParseError) -> Self {
        Self::Mxp(value)
    }
}

impl From<io::Error> for ParseError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}
