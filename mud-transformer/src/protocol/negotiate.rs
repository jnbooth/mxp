use std::fmt;

use crate::TransformerConfig;

pub(crate) trait Negotiate {
    const CODE: u8;

    fn negotiate<W: fmt::Write>(self, f: W, config: &TransformerConfig) -> fmt::Result;
}
