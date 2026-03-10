use flagset::FlagSet;

use crate::argument::{Decoder, Scan};
use crate::keyword::MxpKeyword;
use crate::parser::Error;

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
