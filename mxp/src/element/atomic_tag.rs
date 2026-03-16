use std::borrow::Cow;

use super::action::Action;
use super::action_kind::ActionKind;
use crate::arguments::Arguments;
use crate::case_insensitive::to_ascii_lowercase;
use crate::parse::Decoder;
use crate::{Error, ErrorKind};

/// Atomic MXP tags, such as `<A>`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AtomicTag {
    /// Tag name, such as `"A"`.
    pub name: &'static str,
    /// Action applied by the tag.
    pub action: ActionKind,
    /// Arguments supported by the tag, such as `"href"`.
    pub args: &'static [&'static str],
}

macro_rules! tag {
    ($l:literal, $i:ident) => {
        Self {
            name: $l,
            action: ActionKind::$i,
            args: &[],
        }
    };
    ($l:literal, $i:ident, $($a:literal),+ $(,)?) => {
        Self {
            name: $l,
            action: ActionKind::$i,
            args: &[$($a),+],
        }
    };
}

impl AtomicTag {
    /// Lists all atomic tags supported by the `mxp` crate.
    pub const fn supported() -> &'static [Self] {
        Self::SUPPORTED
    }

    /// Resolves an `AtomicTag` into an [`Action`] by decoding arguments and supplying them to the
    /// tag's definition.
    pub fn decode<'a, D: Decoder>(
        &self,
        args: &'a Arguments<'a>,
        decoder: D,
    ) -> crate::Result<Action<Cow<'a, str>>> {
        self.check_arguments(args)?;
        Action::decode(self.action, args.scan(decoder))
    }

    /// Returns `true` if this library's definition of the tag supports a specific argument.
    ///
    /// Case-insensitive.
    ///
    /// # Examples
    ///
    /// ```
    /// const COLOR: &mxp::AtomicTag = mxp::AtomicTag::well_known("color").unwrap();
    /// assert!(COLOR.supports("fore"));
    /// assert!(!COLOR.supports("invalid_arg"));
    /// ```
    pub const fn supports(&self, arg: &str) -> bool {
        let mut i = 0;
        while i < self.args.len() {
            if self.args[i].eq_ignore_ascii_case(arg) {
                return true;
            }
            i += 1;
        }
        false
    }

    /// Ensures all named arguments are supported.
    pub(crate) fn check_arguments<S: AsRef<str>>(&self, args: &Arguments<S>) -> crate::Result<()> {
        match args.keys().find(|arg| !self.supports(arg.as_str())) {
            None => Ok(()),
            Some(arg) => Err(Error::new(arg.as_str(), ErrorKind::UnexpectedArgument)),
        }
    }

    /// Returns an `AtomicTag` if `name` is a well-known MXP tag, such as `"A"` or `"image"`.
    ///
    /// Case-insensitive.
    ///
    /// # Examples
    ///
    /// ```
    /// let em = mxp::AtomicTag::well_known("em").unwrap();
    /// assert_eq!(em.action, mxp::ActionKind::Italic);
    /// ```
    pub const fn well_known(name: &str) -> Option<&'static Self> {
        const MAX_LEN: usize = {
            let tags = AtomicTag::SUPPORTED;
            let mut max_len = 0;
            let mut i = 0;
            while i < tags.len() {
                let len = tags[i].name.len();
                if len > max_len {
                    max_len = len;
                }
                i += 1;
            }
            max_len
        };

        let mut buf = [0; MAX_LEN];
        let Some(name_lower) = to_ascii_lowercase(name.as_bytes(), &mut buf) else {
            return None;
        };

        match name_lower {
            b"a" => Some(&Self::A),
            b"b" => Some(&Self::B),
            b"bold" => Some(&Self::BOLD),
            b"br" => Some(&Self::BR),
            b"c" => Some(&Self::C),
            b"color" => Some(&Self::COLOR),
            b"dest" => Some(&Self::DEST),
            b"destination" => Some(&Self::DESTINATION),
            b"em" => Some(&Self::EM),
            b"expire" => Some(&Self::EXPIRE),
            b"filter" => Some(&Self::FILTER),
            b"font" => Some(&Self::FONT),
            b"frame" => Some(&Self::FRAME),
            b"gauge" => Some(&Self::GAUGE),
            b"h" => Some(&Self::H),
            b"h1" => Some(&Self::H1),
            b"h2" => Some(&Self::H2),
            b"h3" => Some(&Self::H3),
            b"h4" => Some(&Self::H4),
            b"h5" => Some(&Self::H5),
            b"h6" => Some(&Self::H6),
            b"high" => Some(&Self::HIGH),
            b"hr" => Some(&Self::HR),
            b"i" => Some(&Self::I),
            b"image" => Some(&Self::IMAGE),
            b"italic" => Some(&Self::ITALIC),
            b"music" => Some(&Self::MUSIC),
            b"mxp" => Some(&Self::MXP),
            b"nobr" => Some(&Self::NOBR),
            b"p" => Some(&Self::P),
            b"pass" => Some(&Self::PASS),
            b"password" => Some(&Self::PASSWORD),
            b"relocate" => Some(&Self::RELOCATE),
            b"reset" => Some(&Self::RESET),
            b"s" => Some(&Self::S),
            b"sbr" => Some(&Self::SBR),
            b"send" => Some(&Self::SEND),
            b"small" => Some(&Self::SMALL),
            b"sound" => Some(&Self::SOUND),
            b"stat" => Some(&Self::STAT),
            b"strike" => Some(&Self::STRIKE),
            b"strikeout" => Some(&Self::STRIKEOUT),
            b"strong" => Some(&Self::STRONG),
            b"support" => Some(&Self::SUPPORT),
            b"tt" => Some(&Self::TT),
            b"u" => Some(&Self::U),
            b"underline" => Some(&Self::UNDERLINE),
            b"user" => Some(&Self::USER),
            b"username" => Some(&Self::USERNAME),
            b"v" => Some(&Self::V),
            b"var" => Some(&Self::VAR),
            b"version" => Some(&Self::VERSION),
            _ => None,
        }
    }

    const A: AtomicTag = tag!("a", Hyperlink, "href", "hint", "expire");
    const B: AtomicTag = tag!("b", Bold);
    const BOLD: AtomicTag = tag!("bold", Bold);
    const BR: AtomicTag = tag!("br", Br);
    const C: AtomicTag = tag!("c", Color, "fore", "back");
    const COLOR: AtomicTag = tag!("color", Color, "fore", "back");
    const DEST: AtomicTag = tag!("dest", Dest);
    const DESTINATION: AtomicTag = tag!("destination", Dest);
    const EM: AtomicTag = tag!("em", Italic);
    const EXPIRE: AtomicTag = tag!("expire", Expire);
    const FILTER: AtomicTag = tag!("filter", Filter);
    const FONT: AtomicTag = tag!("font", Font, "face", "size", "color", "back");
    const FRAME: AtomicTag = tag!(
        "frame",
        Frame,
        "name",
        "action",
        "title",
        "internal",
        "align",
        "left",
        "top",
        "width",
        "height",
        "scrolling",
        "floating"
    );
    const GAUGE: AtomicTag = tag!("gauge", Gauge);
    const H: AtomicTag = tag!("h", Highlight);
    const H1: AtomicTag = tag!("h1", H1);
    const H2: AtomicTag = tag!("h2", H2);
    const H3: AtomicTag = tag!("h3", H3);
    const H4: AtomicTag = tag!("h4", H4);
    const H5: AtomicTag = tag!("h5", H5);
    const H6: AtomicTag = tag!("h6", H6);
    const HIGH: AtomicTag = tag!("high", Highlight);
    const HR: AtomicTag = tag!("hr", Hr);
    const I: AtomicTag = tag!("i", Italic);
    const IMAGE: AtomicTag = tag!(
        "image", Image, "url", "fname", "t", "h", "w", "hspace", "vspace", "align", "ismap"
    );
    const ITALIC: AtomicTag = tag!("italic", Italic);
    const MUSIC: AtomicTag = tag!("music", Music, "fname", "v", "l", "c", "t", "u");
    const MXP: AtomicTag = tag!("mxp", Mxp, "off");
    const NOBR: AtomicTag = tag!("nobr", NoBr);
    const P: AtomicTag = tag!("p", P);
    const PASS: AtomicTag = tag!("pass", Password);
    const PASSWORD: AtomicTag = tag!("password", Password);
    const RELOCATE: AtomicTag = tag!("relocate", Relocate);
    const RESET: AtomicTag = tag!("reset", Reset);
    const S: AtomicTag = tag!("s", Strikeout);
    const SBR: AtomicTag = tag!("sbr", SBr);
    const SEND: AtomicTag = tag!("send", Send, "href", "hint", "prompt", "expire");
    const SMALL: AtomicTag = tag!("small", Small);
    const SOUND: AtomicTag = tag!("sound", Sound, "fname", "v", "l", "p", "t", "u");
    const STAT: AtomicTag = tag!("stat", Stat);
    const STRIKE: AtomicTag = tag!("strike", Strikeout);
    const STRIKEOUT: AtomicTag = tag!("strikeout", Strikeout);
    const STRONG: AtomicTag = tag!("strong", Bold);
    const SUPPORT: AtomicTag = tag!("support", Support);
    const TT: AtomicTag = tag!("tt", Tt);
    const U: AtomicTag = tag!("u", Underline);
    const UNDERLINE: AtomicTag = tag!("underline", Underline);
    const USER: AtomicTag = tag!("user", User);
    const USERNAME: AtomicTag = tag!("username", User);
    const V: AtomicTag = tag!("v", Var);
    const VAR: AtomicTag = tag!("var", Var);
    const VERSION: AtomicTag = tag!("version", Version);

    const SUPPORTED: &[AtomicTag] = &[
        Self::A,
        Self::B,
        Self::BOLD,
        Self::BR,
        Self::C,
        Self::COLOR,
        Self::DEST,
        Self::DESTINATION,
        Self::EM,
        Self::EXPIRE,
        Self::FILTER,
        Self::FONT,
        Self::FRAME,
        Self::GAUGE,
        Self::H,
        Self::H1,
        Self::H2,
        Self::H3,
        Self::H4,
        Self::H5,
        Self::H6,
        Self::HIGH,
        Self::HR,
        Self::I,
        Self::IMAGE,
        Self::ITALIC,
        Self::MUSIC,
        Self::MXP,
        Self::NOBR,
        Self::P,
        Self::PASS,
        Self::PASSWORD,
        Self::RELOCATE,
        Self::RESET,
        Self::S,
        Self::SBR,
        Self::SEND,
        Self::SMALL,
        Self::SOUND,
        Self::STAT,
        Self::STRIKE,
        Self::STRIKEOUT,
        Self::STRONG,
        Self::SUPPORT,
        Self::TT,
        Self::U,
        Self::UNDERLINE,
        Self::USER,
        Self::USERNAME,
        Self::V,
        Self::VAR,
        Self::VERSION,
    ];
}
