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
    /// Fragment causes a new line to begin, resetting most ANSI effects and flushing text.
    pub const fn resets_line(&self) -> bool {
        matches!(
            self,
            Self::Hr
                | Self::LineBreak
                | Self::PageBreak
                | Self::Control(ControlFragment::CarriageReturn)
        )
    }

    /// Fragment takes up space inside a line of text.
    pub const fn is_line_content(&self) -> bool {
        matches!(self, Self::Image(_) | Self::Text(_))
    }

    /// Fragment does not target a specific window, so it doesn't need to be associated with an
    /// [`Output::window`] MXP tag.
    pub(super) const fn is_windowless(&self) -> bool {
        matches!(self, Self::Mxp(_) | Self::Telnet(_))
    }

    /// Fragment does not require the current line of text to be flushed to output as a text
    /// fragment before handling.
    ///
    /// For non-text fragments, this means the fragment will be processed out of order, in that it
    /// will be sent before the current line of text. This is beneficial because it means text
    /// fragments will, as much as possible, contain full lines of text (i.e. terminated by a
    /// newline). Therefore, any fragment that depends on its position in the text should flush.
    pub(super) const fn should_flush(&self) -> bool {
        !matches!(
            self,
            Self::Mxp(_)
                | Self::Control(
                    ControlFragment::Beep
                        | ControlFragment::SetDisconnectDelay(_)
                        | ControlFragment::ManipulateSelection(..)
                        | ControlFragment::SetIconLabel(_)
                        | ControlFragment::SetKeyClickVolume(_)
                        | ControlFragment::SetMarginVolume(_)
                        | ControlFragment::SetLed(..)
                        | ControlFragment::SetRefreshRate(_)
                        | ControlFragment::SetScrollSpeed(_)
                        | ControlFragment::SetTitle(_)
                        | ControlFragment::SetWarningVolume(_)
                        | ControlFragment::StyleCursor(_)
                        | ControlFragment::TimeOfDay { .. }
                )
        )
    }
}
