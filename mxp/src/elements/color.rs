use crate::argument::{Decoder, Scan};
use crate::color::RgbColor;
use crate::parser::Error;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Color {
    pub fore: Option<RgbColor>,
    pub back: Option<RgbColor>,
}

impl<'a, D> TryFrom<Scan<'a, D>> for Color
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
