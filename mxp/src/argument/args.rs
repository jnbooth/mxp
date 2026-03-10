use std::borrow::Cow;

use flagset::FlagSet;

use super::scan::{Decoder, ExpectArg, Scan};
use crate::color::RgbColor;
use crate::keyword::{EntityKeyword, MxpKeyword};
use crate::parser::Error;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ColorArgs {
    pub fore: Option<RgbColor>,
    pub back: Option<RgbColor>,
}

impl<'a, D> TryFrom<Scan<'a, D>> for ColorArgs
where
    D: Decoder,
{
    type Error = Error;

    fn try_from(mut scanner: Scan<'a, D>) -> crate::Result<Self> {
        Ok(Self {
            fore: scanner
                .next_or("fore")?
                .and_then(|fore| RgbColor::named(&fore)),
            back: scanner
                .next_or("back")?
                .and_then(|back| RgbColor::named(&back)),
        })
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct MxpArgs {
    pub keywords: FlagSet<MxpKeyword>,
}

impl<'a, D> TryFrom<Scan<'a, D>> for MxpArgs
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

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct SupportArgs<S> {
    pub questions: Vec<S>,
}

impl<'a, D> TryFrom<Scan<'a, D>> for SupportArgs<Cow<'a, str>>
where
    D: Decoder,
{
    type Error = Error;

    fn try_from(mut scanner: Scan<'a, D>) -> Result<Self, Self::Error> {
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

impl<'a, D> TryFrom<Scan<'a, D>> for VarArgs<Cow<'a, str>>
where
    D: Decoder,
{
    type Error = Error;

    fn try_from(scanner: Scan<'a, D>) -> crate::Result<Self> {
        let mut scanner = scanner.with_keywords();
        Ok(Self {
            variable: scanner.next()?.expect_some("variable")?,
            keywords: scanner.into_keywords(),
        })
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct VersionArgs<S> {
    pub styleversion: Option<S>,
}

impl<'a, D> TryFrom<Scan<'a, D>> for VersionArgs<Cow<'a, str>>
where
    D: Decoder,
{
    type Error = Error;

    fn try_from(mut scanner: Scan<'a, D>) -> crate::Result<Self> {
        Ok(Self {
            styleversion: scanner.next()?,
        })
    }
}
