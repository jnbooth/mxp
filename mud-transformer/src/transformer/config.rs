use std::collections::HashSet;

use enumeration::Enum;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Enum)]
pub enum UseMxp {
    Command,
    Query,
    Always,
    Never,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransformerConfig {
    pub use_mxp: UseMxp,
    pub disable_compression: bool,
    pub terminal_identification: String,
    pub app_name: String,
    pub version: String,
    pub player: String,
    pub password: String,
    pub convert_ga_to_newline: bool,
    pub no_echo_off: bool,
    pub naws: bool,
    pub disable_utf8: bool,
    pub ignore_mxp_colors: bool,
    pub will: HashSet<u8>,
}

impl Default for TransformerConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl TransformerConfig {
    pub fn new() -> Self {
        Self {
            app_name: String::new(),
            version: String::new(),
            player: String::new(),
            password: String::new(),
            convert_ga_to_newline: false,
            no_echo_off: false,
            naws: false,
            disable_utf8: false,
            disable_compression: false,
            use_mxp: UseMxp::Command,
            terminal_identification: String::new(),
            ignore_mxp_colors: false,
            will: HashSet::new(),
        }
    }
}
