use std::borrow::Cow;

use flagset::FlagSet;

use super::scan::{Decoder, ExpectArg, Scan};
use crate::color::RgbColor;
use crate::keyword::{EntityKeyword, MxpKeyword};
use crate::parser::Error;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ColorArgs {
    pub fore: Option<RgbColor>,
    pub back: Option<RgbColor>,
}

impl<'a, D, S> TryFrom<Scan<'a, D, S>> for ColorArgs
where
    D: Decoder,
    S: AsRef<str>,
{
    type Error = Error;

    fn try_from(mut scanner: Scan<'a, D, S>) -> crate::Result<Self> {
        Ok(Self {
            fore: scanner
                .next_or("fore")?
                .and_then(|fore| RgbColor::named(fore.as_ref())),
            back: scanner
                .next_or("back")?
                .and_then(|back| RgbColor::named(back.as_ref())),
        })
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct MxpArgs {
    pub keywords: FlagSet<MxpKeyword>,
}

impl<'a, D, S> TryFrom<Scan<'a, D, S>> for MxpArgs
where
    D: Decoder,
    S: AsRef<str>,
{
    type Error = Error;

    fn try_from(scanner: Scan<'a, D, S>) -> crate::Result<Self> {
        let scanner = scanner.with_keywords();
        Ok(Self {
            keywords: scanner.into_keywords(),
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct SupportArgs<S> {
    pub questions: Vec<S>,
}

impl<'a, D, S> TryFrom<Scan<'a, D, S>> for SupportArgs<Cow<'a, str>>
where
    D: Decoder,
    S: AsRef<str>,
{
    type Error = Error;

    fn try_from(mut scanner: Scan<'a, D, S>) -> Result<Self, Self::Error> {
        let mut questions = Vec::with_capacity(scanner.len());
        while let Some(question) = scanner.next()? {
            questions.push(question);
        }
        Ok(Self { questions })
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct VarArgs<S> {
    pub variable: S,
    pub keywords: FlagSet<EntityKeyword>,
}

impl<'a, D, S> TryFrom<Scan<'a, D, S>> for VarArgs<Cow<'a, str>>
where
    D: Decoder,
    S: AsRef<str>,
{
    type Error = Error;

    fn try_from(scanner: Scan<'a, D, S>) -> crate::Result<Self> {
        let mut scanner = scanner.with_keywords();
        Ok(Self {
            variable: scanner.next()?.expect_some("variable")?,
            keywords: scanner.into_keywords(),
        })
    }
}
