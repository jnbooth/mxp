use mxp::RgbColor;
use std::collections::HashSet;

use enumeration::{Enum, EnumSet};
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransformerConfig {
    pub app_name: String,
    pub colors: Vec<RgbColor>,
    pub convert_ga_to_newline: bool,
    pub disable_compression: bool,
    pub disable_utf8: bool,
    pub ignore_mxp_colors: bool,
    pub naws: bool,
    pub no_echo_off: bool,
    pub password: String,
    pub player: String,
    pub terminal_identification: String,
    pub unsupported_actions: EnumSet<mxp::ActionKind>,
    pub use_mxp: UseMxp,
    pub version: String,
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
            colors: Vec::new(),
            convert_ga_to_newline: false,
            disable_compression: false,
            disable_utf8: false,
            ignore_mxp_colors: false,
            naws: false,
            no_echo_off: false,
            password: String::new(),
            player: String::new(),
            terminal_identification: String::new(),
            unsupported_actions: EnumSet::new(),
            use_mxp: UseMxp::Command,
            version: String::new(),
            will: HashSet::new(),
        }
    }
}
