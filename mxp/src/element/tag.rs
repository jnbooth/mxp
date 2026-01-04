use casefold::ascii::CaseFold;

use super::action::ActionKind;

/// Atomic MXP tags that we recognise, e.g. <b>.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tag {
    /// Tag name, e.g. bold
    pub name: &'static str,
    /// Its action.
    pub action: ActionKind,
    /// Supported arguments, e.g. href, hint
    pub args: &'static [&'static CaseFold<str>],
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
            args: &[$(CaseFold::borrow($a)),+],
        }
    };
}

impl Tag {
    pub(crate) const fn new(
        name: &'static str,
        action: ActionKind,
        args: &'static [&'static CaseFold<str>],
    ) -> Self {
        Self { name, action, args }
    }

    pub const fn supported() -> &'static [Self] {
        Self::SUPPORTED
    }

    pub const fn well_known(name: &str) -> Option<&'static Self> {
        const MAX_LEN: usize = {
            let tags = Tag::SUPPORTED;
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

        let Some((name_lower, _)) = buf.split_at_mut_checked(name.len()) else {
            return None;
        };

        name_lower.copy_from_slice(name.as_bytes());
        name_lower.make_ascii_lowercase();

        match &*name_lower {
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

    const A: Tag = tag!("a", Hyperlink, "href", "hint", "expire");
    const B: Tag = tag!("b", Bold);
    const BOLD: Tag = tag!("bold", Bold);
    const BR: Tag = tag!("br", Br);
    const C: Tag = tag!("c", Color, "fore", "back");
    const COLOR: Tag = tag!("color", Color, "fore", "back");
    const DEST: Tag = tag!("dest", Dest);
    const DESTINATION: Tag = tag!("destination", Dest);
    const EM: Tag = tag!("em", Italic);
    const EXPIRE: Tag = tag!("expire", Expire);
    const FILTER: Tag = tag!("filter", Filter);
    const FONT: Tag = tag!("font", Font, "face", "size", "color", "back");
    const GAUGE: Tag = tag!("gauge", Gauge);
    const H: Tag = tag!("h", Highlight);
    const H1: Tag = tag!("h1", H1);
    const H2: Tag = tag!("h2", H2);
    const H3: Tag = tag!("h3", H3);
    const H4: Tag = tag!("h4", H4);
    const H5: Tag = tag!("h5", H5);
    const H6: Tag = tag!("h6", H6);
    const HIGH: Tag = tag!("high", Highlight);
    const HR: Tag = tag!("hr", Hr);
    const I: Tag = tag!("i", Italic);
    const IMAGE: Tag = tag!(
        "image", Image, "url", "fname", "t", "h", "w", "hspace", "vspace", "align", "ismap"
    );
    const ITALIC: Tag = tag!("italic", Italic);
    const MUSIC: Tag = tag!("music", Music, "fname", "v", "l", "c", "t", "u");
    const MXP: Tag = tag!("mxp", Mxp, "off");
    const NOBR: Tag = tag!("nobr", NoBr);
    const P: Tag = tag!("p", P);
    const PASS: Tag = tag!("pass", Password);
    const PASSWORD: Tag = tag!("password", Password);
    const RELOCATE: Tag = tag!("relocate", Relocate);
    const RESET: Tag = tag!("reset", Reset);
    const S: Tag = tag!("s", Strikeout);
    const SBR: Tag = tag!("sbr", SBr);
    const SEND: Tag = tag!("send", Send, "href", "hint", "prompt", "expire");
    const SMALL: Tag = tag!("small", Small);
    const SOUND: Tag = tag!("sound", Sound, "fname", "v", "l", "p", "t", "u");
    const STAT: Tag = tag!("stat", Stat);
    const STRIKE: Tag = tag!("strike", Strikeout);
    const STRIKEOUT: Tag = tag!("strikeout", Strikeout);
    const STRONG: Tag = tag!("strong", Bold);
    const SUPPORT: Tag = tag!("support", Support);
    const TT: Tag = tag!("tt", Tt);
    const U: Tag = tag!("u", Underline);
    const UNDERLINE: Tag = tag!("underline", Underline);
    const USER: Tag = tag!("user", User);
    const USERNAME: Tag = tag!("username", User);
    const V: Tag = tag!("v", Var);
    const VAR: Tag = tag!("var", Var);
    const VERSION: Tag = tag!("version", Version);

    const SUPPORTED: &[Tag] = &[
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
