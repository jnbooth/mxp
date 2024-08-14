use std::num::NonZeroU8;

use super::atom::Atom;
use super::link::Link;
use crate::argument::scan::{
    AfkArgs, ColorArgs, Decoder, ExpireArgs, FontArgs, HyperlinkArgs, ImageArgs, Scan, SendArgs,
    VarArgs,
};
use crate::argument::{FgColor, XchMode};
use crate::color::RgbColor;
use crate::keyword::{EntityKeyword, MxpKeyword};
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
    /// Soft line break
    SBr,
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
        face: Option<S>,
        size: Option<NonZeroU8>,
        fgcolor: Option<FgColor<S>>,
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
        is_map: bool,
    },
    /// sound/image filter
    Filter,
    /// Hyperlink or send prompt (secure)
    Link(Link),
    /// Hard Line break (secure)
    Br,
    /// heading (secure)
    Heading(Heading),
    /// Horizontal rule (secure)
    Hr,
    /// non-breaking newline
    NoBr,
    /// Soft linebreak
    SBr,
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
        keywords: EnumSet<EntityKeyword>,
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
    Expire {
        name: Option<S>,
    },

    /// close all open tags
    Reset,
    /// MXP command (eg. MXP OFF)
    Mxp {
        keywords: EnumSet<MxpKeyword>,
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
        is_map: bool,
    },
    XchPage,
    XchPane,
}

impl<S: AsRef<str>> Action<S> {
    pub fn new<'a, D: Decoder>(action: ActionType, mut scanner: Scan<'a, D>) -> crate::Result<Self>
    where
        D: Decoder<Output<'a> = S>,
    {
        Ok(match action {
            ActionType::Send => {
                let args = SendArgs::try_from(scanner)?;
                Self::Link(args.into())
            }
            ActionType::Bold => Self::Bold,
            ActionType::Underline => Self::Underline,
            ActionType::Italic => Self::Italic,
            ActionType::Color => {
                let ColorArgs { fore, back } = scanner.try_into()?;
                Self::Color { fore, back }
            }
            ActionType::Version => Self::Version,
            ActionType::Font => {
                let FontArgs {
                    face,
                    size,
                    fgcolor,
                    bgcolor,
                } = scanner.try_into()?;
                Self::Font {
                    face,
                    size,
                    fgcolor,
                    bgcolor,
                }
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
                    is_map,
                } = scanner.try_into()?;
                Self::Image {
                    fname,
                    url,
                    xch_mode,
                    is_map,
                }
            }
            ActionType::Filter => Self::Filter,
            ActionType::Hyperlink => {
                let args = HyperlinkArgs::try_from(scanner)?;
                Self::Link(args.into())
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
            ActionType::SBr => Self::SBr,
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
                let VarArgs { keywords, variable } = scanner.try_into()?;
                Self::Var { keywords, variable }
            }
            ActionType::Afk => {
                let AfkArgs { challenge } = scanner.try_into()?;
                Self::Afk { challenge }
            }
            ActionType::Gauge => Self::Gauge,
            ActionType::Stat => Self::Stat,
            ActionType::Expire => {
                let ExpireArgs { name } = scanner.try_into()?;
                Self::Expire { name }
            }
            ActionType::Reset => Self::Reset,
            ActionType::Mxp => Self::Mxp {
                keywords: scanner.with_keywords().into_keywords(),
            },
            ActionType::Support => {
                let mut questions = Vec::with_capacity(scanner.len());
                while let Some(question) = scanner.next()? {
                    questions.push(question);
                }
                let mut supported = Vec::new();
                Atom::fmt_supported(&mut supported, &questions);
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
                    is_map,
                } = scanner.try_into()?;
                Self::Img {
                    fname,
                    url,
                    xch_mode,
                    is_map,
                }
            }
            ActionType::XchPage => Self::XchPage,
            ActionType::XchPane => Self::XchPane,
        })
    }
}
