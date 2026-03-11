use std::str::FromStr;

use flagset::FlagSet;

use crate::keyword::MxpKeyword;
use crate::parse::{Decoder, Error, Scan};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Mxp {
    pub keywords: FlagSet<MxpKeyword>,
}

impl<'a, D> TryFrom<Scan<'a, D>> for Mxp
where
    D: Decoder,
{
    type Error = Error;

    fn try_from(scanner: Scan<'a, D>) -> crate::Result<Self> {
        let scanner = scanner.with_keywords();
        Ok(Self {
            keywords: scanner.into_keywords(),
        })
    }
}

impl FromStr for Mxp {
    type Err = crate::parse::FromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::parse::parse_element(s, crate::ActionKind::Mxp)
    }
}
