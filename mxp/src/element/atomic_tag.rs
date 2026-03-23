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
        Action::decode(self.action, args.scan().with_decoder(decoder))
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

    const A: AtomicTag = tag!("A", Hyperlink, "href", "hint", "expire");
    const B: AtomicTag = tag!("B", Bold);
    const BOLD: AtomicTag = tag!("BOLD", Bold);
    const BR: AtomicTag = tag!("BR", Br);
    const C: AtomicTag = tag!("C", Color, "fore", "back");
    const COLOR: AtomicTag = tag!("COLOR", Color, "fore", "back");
    const DEST: AtomicTag = tag!("DEST", Dest, "x", "y", "eof", "eol");
    const DESTINATION: AtomicTag = tag!("DESTINATION", Dest, "x", "y", "eof", "eol");
    const EM: AtomicTag = tag!("EM", Italic);
    const EXPIRE: AtomicTag = tag!("EXPIRE", Expire);
    const FILTER: AtomicTag = tag!("FILTER", Filter, "src", "dest", "name", "proc");
    const FONT: AtomicTag = tag!("FONT", Font, "face", "size", "color", "back");
    const FRAME: AtomicTag = tag!(
        "FRAME",
        Frame,
        "name",
        "action",
        "title",
        "align",
        "left",
        "top",
        "width",
        "height",
        "scrolling",
        "internal",
        "external",
        "persistent",
        "floating",
        "dock",
    );
    const GAUGE: AtomicTag = tag!("GAUGE", Gauge, "max", "caption", "color");
    const H: AtomicTag = tag!("H", Highlight);
    const H1: AtomicTag = tag!("H1", H1);
    const H2: AtomicTag = tag!("H2", H2);
    const H3: AtomicTag = tag!("H3", H3);
    const H4: AtomicTag = tag!("H4", H4);
    const H5: AtomicTag = tag!("H5", H5);
    const H6: AtomicTag = tag!("H6", H6);
    const HIGH: AtomicTag = tag!("HIGH", Highlight);
    const HR: AtomicTag = tag!("HR", Hr);
    const I: AtomicTag = tag!("I", Italic);
    const IMAGE: AtomicTag = tag!(
        "IMAGE", Image, "fname", "url", "t", "h", "w", "hspace", "vspace", "align", "ismap"
    );
    const ITALIC: AtomicTag = tag!("ITALIC", Italic);
    const MUSIC: AtomicTag = tag!("MUSIC", Music, "fname", "v", "l", "c", "t", "u");
    const MXP: AtomicTag = tag!("MXP", Mxp, "off");
    const NOBR: AtomicTag = tag!("NOBR", NoBr);
    const P: AtomicTag = tag!("P", P);
    const PASS: AtomicTag = tag!("PASS", Password);
    const PASSWORD: AtomicTag = tag!("PASSWORD", Password);
    const RELOCATE: AtomicTag = tag!("RELOCATE", Relocate, "quiet");
    const RESET: AtomicTag = tag!("RESET", Reset);
    const S: AtomicTag = tag!("S", Strikeout);
    const SBR: AtomicTag = tag!("SBR", SBr);
    const SEND: AtomicTag = tag!("SEND", Send, "href", "hint", "prompt", "expire");
    const SMALL: AtomicTag = tag!("SMALL", Small);
    const SOUND: AtomicTag = tag!("SOUND", Sound, "fname", "v", "l", "p", "t", "u");
    const STAT: AtomicTag = tag!("STAT", Stat, "entity", "max", "caption");
    const STRIKE: AtomicTag = tag!("STRIKE", Strikeout);
    const STRIKEOUT: AtomicTag = tag!("STRIKEOUT", Strikeout);
    const STRONG: AtomicTag = tag!("STRONG", Bold);
    const SUPPORT: AtomicTag = tag!("SUPPORT", Support);
    const TT: AtomicTag = tag!("TT", Tt);
    const U: AtomicTag = tag!("U", Underline);
    const UNDERLINE: AtomicTag = tag!("UNDERLINE", Underline);
    const USER: AtomicTag = tag!("USER", User);
    const USERNAME: AtomicTag = tag!("USERNAME", User);
    const V: AtomicTag = tag!("V", Var);
    const VAR: AtomicTag = tag!(
        "VAR", Var, "desc", "private", "publish", "delete", "add", "remove"
    );
    const VERSION: AtomicTag = tag!("VERSION", Version);

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
