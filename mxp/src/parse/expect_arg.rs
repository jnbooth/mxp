use std::num::ParseIntError;
use std::str::FromStr;

use crate::color::RgbColor;
use crate::parse::UnrecognizedVariant;
use crate::{Error, ErrorKind};

pub(crate) trait ExpectArg {
    type Arg;

    fn expect_color(self) -> crate::Result<Option<RgbColor>>
    where
        Self::Arg: AsRef<str>;

    fn expect_some(self, name: &str) -> crate::Result<Self::Arg>;

    fn expect_number<T>(self) -> crate::Result<Option<T>>
    where
        Self::Arg: AsRef<str>,
        T: FromStr<Err = ParseIntError>;

    fn expect_variant<T>(self) -> crate::Result<Option<T>>
    where
        Self::Arg: AsRef<str>,
        T: FromStr<Err = UnrecognizedVariant<T>>;
}

impl<S> ExpectArg for Option<S> {
    type Arg = S;

    fn expect_color(self) -> crate::Result<Option<RgbColor>>
    where
        Self::Arg: AsRef<str>,
    {
        let Some(arg) = self else {
            return Ok(None);
        };
        let arg = arg.as_ref();
        if arg.is_empty() {
            return Ok(None);
        }
        match RgbColor::named(arg) {
            Some(color) => Ok(Some(color)),
            None => Err(Error::new(arg, ErrorKind::UnknownColor)),
        }
    }

    fn expect_some(self, name: &str) -> crate::Result<Self::Arg> {
        match self {
            Some(arg) => Ok(arg),
            None => Err(Error::new(name, ErrorKind::MissingArguments)),
        }
    }

    fn expect_number<T>(self) -> crate::Result<Option<T>>
    where
        Self::Arg: AsRef<str>,
        T: FromStr<Err = ParseIntError>,
    {
        let Some(arg) = self else {
            return Ok(None);
        };
        let arg = arg.as_ref();
        if arg.is_empty() {
            return Ok(None);
        }
        match arg.parse() {
            Ok(parsed) => Ok(Some(parsed)),
            Err(_) => Err(Error::new(arg, ErrorKind::InvalidNumber)),
        }
    }

    fn expect_variant<T>(self) -> crate::Result<Option<T>>
    where
        Self::Arg: AsRef<str>,
        T: FromStr<Err = UnrecognizedVariant<T>>,
    {
        let Some(arg) = self else {
            return Ok(None);
        };
        let arg = arg.as_ref();
        if arg.is_empty() {
            return Ok(None);
        }
        match arg.parse() {
            Ok(parsed) => Ok(Some(parsed)),
            Err(_) => Err(Error::new(arg, ErrorKind::UnexpectedArgument)),
        }
    }
}
