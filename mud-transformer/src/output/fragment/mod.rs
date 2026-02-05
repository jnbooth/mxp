use bytestring::ByteString;

mod control_fragment;
pub use control_fragment::ControlFragment;

mod mxp_fragment;
pub use mxp_fragment::{EntityFragment, MxpFragment};

mod telnet_fragment;
pub use telnet_fragment::{TelnetFragment, TelnetSource, TelnetVerb};

mod text_fragment;
pub use text_fragment::{TextFragment, TextFragmentANSI, TextFragmentHtml};

pub type OutputDrain<'a> = std::vec::Drain<'a, Output>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output {
    pub fragment: OutputFragment,
    pub gag: bool,
    pub window: Option<ByteString>,
}

impl<T> From<T> for Output
where
    T: Into<OutputFragment>,
{
    fn from(value: T) -> Self {
        Self {
            fragment: value.into(),
            gag: false,
            window: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OutputFragment {
    Control(ControlFragment),
    Frame(mxp::Frame),
    Hr,
    Image(mxp::Image),
    LineBreak,
    Mxp(MxpFragment),
    PageBreak,
    Telnet(TelnetFragment),
    Text(TextFragment),
}

impl OutputFragment {
    pub const fn is_newline(&self) -> bool {
        matches!(self, Self::Hr | Self::LineBreak | Self::PageBreak)
    }

    pub const fn is_line_content(&self) -> bool {
        matches!(self, Self::Image(_) | Self::Text(_))
    }

    pub(super) const fn is_windowless(&self) -> bool {
        matches!(self, Self::Mxp(_) | Self::Telnet(_))
    }

    pub(super) const fn should_flush(&self) -> bool {
        match self {
            Self::Control(effect) => effect.should_flush(),
            Self::Mxp(_) | Self::Text(_) => false,
            _ => true,
        }
    }
}
