use enumeration::Enum;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Enum)]
pub enum AutoConnect {
    Mush,
    Mxp,
    Diku,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Enum)]
pub enum UseMxp {
    Command,
    Query,
    Always,
    Never,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransformerConfig {
    pub use_mxp: UseMxp,
    pub disable_compression: bool,
    pub terminal_identification: String,
    pub app_name: String,
    pub version: String,
    pub send_mxp_afk_response: bool,
    pub player: String,
    pub password: String,
    pub connect_method: Option<AutoConnect>,
    pub convert_ga_to_newline: bool,
    pub no_echo_off: bool,
    pub naws: bool,
    pub utf_8: bool,
    pub ignore_mxp_colors: bool,
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
            send_mxp_afk_response: false,
            player: String::new(),
            password: String::new(),
            connect_method: None,
            convert_ga_to_newline: false,
            no_echo_off: false,
            naws: false,
            utf_8: false,
            disable_compression: false,
            use_mxp: UseMxp::Command,
            terminal_identification: "mushclient".to_owned(),
            ignore_mxp_colors: false,
        }
    }
}
