use std::str;

use super::argument::{Arguments, Keyword};
use super::atom::Atom;
use super::link::SendTo;
use super::scanning::{
    AfkArgs, ColorArgs, FgColor, FontArgs, HyperlinkArgs, ImageArgs, SendArgs, VarArgs, XchMode,
};
use crate::color::RgbColor;
use enumeration::{Enum, EnumSet};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Enum)]
pub enum ActionType {
    /// eg. <send href="go west"> west
    Send,
    /// bold
    Bold,
    /// underline
    Underline,
    /// italic
    Italic,
    /// eg. <color fore=red back=blue>
    Color,
    /// version request
    Version,
    /// Font appearance
    Font,
    /// play sound
    Sound,
    /// send username
    User,
    /// send password
    Password,
    /// causes a new connect to open
    Relocate,
    /// frame
    Frame,
    /// destination frame
    Dest,
    /// show image
    Image,
    /// sound/image filter
    Filter,
    /// Hyperlink (secure)
    Hyperlink,
    /// Hard Line break (secure)
    Br,
    /// Level 1 heading (secure)
    H1,
    /// Level 2 heading (secure)
    H2,
    /// Level 3 heading (secure)
    H3,
    /// Level 4 heading (secure)
    H4,
    /// Level 5 heading (secure)
    H5,
    /// Level 6 heading (secure)
    H6,
    /// Horizontal rule (secure)
    Hr,
    /// non-breaking newline
    NoBr,
    /// Paragraph break (secure)
    P,
    /// Strikethrough
    Strike,
    /// Client script (secure)
    Script,
    /// Small text
    Small,
    /// Non-proportional font
    Tt,
    /// Unordered list
    Ul,
    /// Ordered list
    Ol,
    /// List item
    Li,
    /// Sample text
    Samp,
    /// Centre text
    Center,
    /// Highlight text
    High,
    /// Set variable
    Var,
    /// AFK - away from keyboard time
    Afk,

    // recent
    /// gauge
    Gauge,
    /// status
    Stat,
    /// expire
    Expire,

    /// close all open tags
    Reset,
    /// MXP command (eg. MXP OFF)
    Mxp,
    /// what commands we support
    Support,

    /// client options set
    SetOption,
    /// server sets option
    RecommendOption,

    // Pueblo
    /// Preformatted text
    Pre,
    Body,
    Head,
    Html,
    Title,
    Img,
    XchPage,
    XchPane,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Enum)]
pub enum Heading {
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Action<S> {
    /// eg. <send href="go west"> west
    Send {
        href: Option<S>,
        hint: Option<S>,
        sendto: SendTo,
    },
    /// bold
    Bold,
    /// underline
    Underline,
    /// italic
    Italic,
    /// eg. <color fore=red back=blue>
    Color {
        fore: Option<RgbColor>,
        back: Option<RgbColor>,
    },
    /// version request
    Version,
    /// font appearance
    Font {
        fgcolor: FgColor<S>,
        bgcolor: Option<RgbColor>,
    },
    /// play sound
    Sound,
    /// send username
    User,
    /// send password
    Password,
    /// causes a new connect to open
    Relocate,
    /// frame
    Frame,
    /// destination frame
    Dest,
    /// show image
    Image {
        fname: Option<S>,
        url: Option<S>,
        xch_mode: Option<XchMode>,
    },
    /// sound/image filter
    Filter,
    /// Hyperlink (secure)
    Hyperlink {
        href: Option<S>,
    },
    /// Hard Line break (secure)
    Br,
    /// heading (secure)
    Heading(Heading),
    /// Horizontal rule (secure)
    Hr,
    /// non-breaking newline
    NoBr,
    /// Paragraph break (secure)
    P,
    /// Strikethrough
    Strike,
    /// Client script (secure)
    Script,
    /// Small text
    Small,
    /// Non-proportional font
    Tt,
    /// Unordered list
    Ul,
    /// Ordered list
    Ol,
    /// List item
    Li,
    /// Sample text
    Samp,
    /// Centre text
    Center,
    /// Highlight text
    High,
    /// Set variable
    Var {
        variable: Option<S>,
    },
    /// AFK - away from keyboard time
    Afk {
        challenge: Option<S>,
    },

    // recent
    /// gauge
    Gauge,
    /// status
    Stat,
    /// expire
    Expire,

    /// close all open tags
    Reset,
    /// MXP command (eg. MXP OFF)
    Mxp {
        keywords: EnumSet<Keyword>,
    },
    /// what commands we support
    Support {
        supported: Vec<u8>,
    },

    /// client options set
    SetOption,
    /// server sets option
    RecommendOption,

    // Pueblo
    /// Preformatted text
    Pre,
    Body,
    Head,
    Html,
    Title,
    Img {
        fname: Option<S>,
        url: Option<S>,
        xch_mode: Option<XchMode>,
    },
    XchPage,
    XchPane,
}

impl<'a> Action<&'a str> {
    pub fn new(action: ActionType, args: &'a Arguments) -> Self {
        match action {
            ActionType::Send => {
                let SendArgs { href, hint, sendto } = args.into();
                Self::Send { href, hint, sendto }
            }
            ActionType::Bold => Self::Bold,
            ActionType::Underline => Self::Underline,
            ActionType::Italic => Self::Italic,
            ActionType::Color => {
                let ColorArgs { fore, back } = args.into();
                Self::Color { fore, back }
            }
            ActionType::Version => Self::Version,
            ActionType::Font => {
                let FontArgs { fgcolor, bgcolor } = args.into();
                Self::Font { fgcolor, bgcolor }
            }
            ActionType::Sound => Self::Sound,
            ActionType::User => Self::User,
            ActionType::Password => Self::Password,
            ActionType::Relocate => Self::Relocate,
            ActionType::Frame => Self::Frame,
            ActionType::Dest => Self::Dest,
            ActionType::Image => {
                let ImageArgs {
                    fname,
                    url,
                    xch_mode,
                } = args.into();
                Self::Image {
                    fname,
                    url,
                    xch_mode,
                }
            }
            ActionType::Filter => Self::Filter,
            ActionType::Hyperlink => {
                let HyperlinkArgs { href } = args.into();
                Self::Hyperlink { href }
            }
            ActionType::Br => Self::Br,
            ActionType::H1 => Self::Heading(Heading::H1),
            ActionType::H2 => Self::Heading(Heading::H2),
            ActionType::H3 => Self::Heading(Heading::H3),
            ActionType::H4 => Self::Heading(Heading::H4),
            ActionType::H5 => Self::Heading(Heading::H5),
            ActionType::H6 => Self::Heading(Heading::H6),
            ActionType::Hr => Self::Hr,
            ActionType::NoBr => Self::NoBr,
            ActionType::P => Self::P,
            ActionType::Strike => Self::Strike,
            ActionType::Script => Self::Script,
            ActionType::Small => Self::Small,
            ActionType::Tt => Self::Tt,
            ActionType::Ul => Self::Ul,
            ActionType::Ol => Self::Ol,
            ActionType::Li => Self::Li,
            ActionType::Samp => Self::Samp,
            ActionType::Center => Self::Center,
            ActionType::High => Self::High,
            ActionType::Var => {
                let VarArgs { variable } = args.into();
                Self::Var { variable }
            }
            ActionType::Afk => {
                let AfkArgs { challenge } = args.into();
                Self::Afk { challenge }
            }
            ActionType::Gauge => Self::Gauge,
            ActionType::Stat => Self::Stat,
            ActionType::Expire => Self::Expire,
            ActionType::Reset => Self::Reset,
            ActionType::Mxp => Self::Mxp {
                keywords: args.keywords(),
            },
            ActionType::Support => {
                let mut supported = Vec::new();
                Atom::fmt_supported(&mut supported, args);
                Self::Support { supported }
            }
            ActionType::SetOption => Self::SetOption,
            ActionType::RecommendOption => Self::RecommendOption,
            ActionType::Pre => Self::Pre,
            ActionType::Body => Self::Body,
            ActionType::Head => Self::Head,
            ActionType::Html => Self::Html,
            ActionType::Title => Self::Title,
            ActionType::Img => {
                let ImageArgs {
                    fname,
                    url,
                    xch_mode,
                } = args.into();
                Self::Img {
                    fname,
                    url,
                    xch_mode,
                }
            }
            ActionType::XchPage => Self::XchPage,
            ActionType::XchPane => Self::XchPane,
        }
    }

    pub fn owned(&self) -> Action<String> {
        match self {
            Action::Send { href, hint, sendto } => Action::Send {
                href: href.map(ToOwned::to_owned),
                hint: hint.map(ToOwned::to_owned),
                sendto: *sendto,
            },
            Action::Bold => Action::Bold,
            Action::Underline => Action::Underline,
            Action::Italic => Action::Italic,
            Action::Color { fore, back } => Action::Color {
                fore: *fore,
                back: *back,
            },
            Action::Version => Action::Version,
            Action::Font { fgcolor, bgcolor } => Action::Font {
                fgcolor: FgColor {
                    inner: fgcolor.inner.to_owned(),
                },
                bgcolor: *bgcolor,
            },
            Action::Sound => Action::Sound,
            Action::User => Action::User,
            Action::Password => Action::Password,
            Action::Relocate => Action::Relocate,
            Action::Frame => Action::Frame,
            Action::Dest => Action::Dest,
            Action::Image {
                fname,
                url,
                xch_mode,
            } => Action::Image {
                fname: fname.map(ToOwned::to_owned),
                url: url.map(ToOwned::to_owned),
                xch_mode: *xch_mode,
            },
            Action::Filter => Action::Filter,
            Action::Hyperlink { href } => Action::Hyperlink {
                href: href.map(ToOwned::to_owned),
            },
            Action::Br => Action::Br,
            Action::Heading(heading) => Action::Heading(*heading),
            Action::Hr => Action::Hr,
            Action::NoBr => Action::NoBr,
            Action::P => Action::P,
            Action::Strike => Action::Strike,
            Action::Script => Action::Script,
            Action::Small => Action::Small,
            Action::Tt => Action::Tt,
            Action::Ul => Action::Ul,
            Action::Ol => Action::Ol,
            Action::Li => Action::Li,
            Action::Samp => Action::Samp,
            Action::Center => Action::Center,
            Action::High => Action::High,
            Action::Var { variable } => Action::Var {
                variable: variable.map(ToOwned::to_owned),
            },
            Action::Afk { challenge } => Action::Afk {
                challenge: challenge.map(ToOwned::to_owned),
            },
            Action::Gauge => Action::Gauge,
            Action::Stat => Action::Stat,
            Action::Expire => Action::Expire,
            Action::Reset => Action::Reset,
            Action::Mxp { keywords } => Action::Mxp {
                keywords: *keywords,
            },
            Action::Support { supported } => Action::Support {
                supported: supported.clone(),
            },
            Action::SetOption => Action::SetOption,
            Action::RecommendOption => Action::RecommendOption,
            Action::Pre => Action::Pre,
            Action::Body => Action::Body,
            Action::Head => Action::Head,
            Action::Html => Action::Html,
            Action::Title => Action::Title,
            Action::Img {
                fname,
                url,
                xch_mode,
            } => Action::Img {
                fname: fname.map(ToOwned::to_owned),
                url: url.map(ToOwned::to_owned),
                xch_mode: *xch_mode,
            },
            Action::XchPage => Action::XchPage,
            Action::XchPane => Action::XchPane,
        }
    }
}
