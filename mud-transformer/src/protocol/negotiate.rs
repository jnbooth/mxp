use std::fmt::Display;

use crate::TransformerConfig;

pub(crate) trait Negotiate {
    const CODE: u8;

    type Output<'a>: Display;

    fn negotiate(self, config: &TransformerConfig) -> Self::Output<'_>;
}
