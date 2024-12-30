use std::borrow::Cow;

use super::scan::{Decoder, ExpectArg, Scan};
use crate::color::RgbColor;
use crate::keyword::{EntityKeyword, MxpKeyword};
use crate::parser::Error;
use enumeration::EnumSet;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ColorArgs {
    pub fore: Option<RgbColor>,
    pub back: Option<RgbColor>,
}

impl<'a, D: Decoder, S: AsRef<str>> TryFrom<Scan<'a, D, S>> for ColorArgs {
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

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MxpArgs {
    pub keywords: EnumSet<MxpKeyword>,
}

impl<'a, D: Decoder, S: AsRef<str>> TryFrom<Scan<'a, D, S>> for MxpArgs {
    type Error = Error;

    fn try_from(scanner: Scan<'a, D, S>) -> crate::Result<Self> {
        let scanner = scanner.with_keywords();
        Ok(Self {
            keywords: scanner.into_keywords(),
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SupportArgs<S> {
    pub questions: Vec<S>,
}

impl<'a, D: Decoder, S: AsRef<str>> TryFrom<Scan<'a, D, S>> for SupportArgs<Cow<'a, str>> {
    type Error = Error;

    fn try_from(mut scanner: Scan<'a, D, S>) -> Result<Self, Self::Error> {
        let mut questions = Vec::with_capacity(scanner.len());
        while let Some(question) = scanner.next()? {
            questions.push(question);
        }
        Ok(Self { questions })
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VarArgs<S> {
    pub variable: S,
    pub keywords: EnumSet<EntityKeyword>,
}

impl<'a, D: Decoder, S: AsRef<str>> TryFrom<Scan<'a, D, S>> for VarArgs<Cow<'a, str>> {
    type Error = Error;

    fn try_from(scanner: Scan<'a, D, S>) -> crate::Result<Self> {
        let mut scanner = scanner.with_keywords();
        Ok(Self {
            variable: scanner.next()?.expect_some("variable")?,
            keywords: scanner.into_keywords(),
        })
    }
}
