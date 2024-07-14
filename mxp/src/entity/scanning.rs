use super::argument::{Arguments, Keyword};
use super::link::SendTo;
use std::str;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ColorArgs<'a> {
    pub fore: Option<&'a str>,
    pub back: Option<&'a str>,
}

impl<'a> From<&'a Arguments> for ColorArgs<'a> {
    fn from(args: &'a Arguments) -> Self {
        let mut scanner = args.scan();
        Self {
            fore: scanner.next_or(&["fore"]),
            back: scanner.next_or(&["back"]),
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SendArgs<'a> {
    pub href: Option<&'a str>,
    pub hint: Option<&'a str>,
    pub sendto: SendTo,
}

impl<'a> From<&'a Arguments> for SendArgs<'a> {
    fn from(args: &'a Arguments) -> Self {
        let mut scanner = args.scan();
        Self {
            href: scanner.next_or(&["href", "xch_cmd"]),
            hint: scanner.next_or(&["hint", "xch_hint"]),
            sendto: if args.has_keyword(Keyword::Prompt) {
                SendTo::Input
            } else {
                SendTo::World
            },
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HyperlinkArgs<'a> {
    pub href: Option<&'a str>,
}

impl<'a> From<&'a Arguments> for HyperlinkArgs<'a> {
    fn from(args: &'a Arguments) -> Self {
        let mut scanner = args.scan();
        Self {
            href: scanner.next_or(&["href"]),
        }
    }
}

#[derive(Clone, Debug)]
pub struct FontArgs<'a> {
    pub fgcolor: str::Split<'a, char>,
    pub bgcolor: Option<&'a str>,
}

impl<'a> From<&'a Arguments> for FontArgs<'a> {
    fn from(args: &'a Arguments) -> Self {
        let mut scanner = args.scan();
        Self {
            fgcolor: scanner
                .next_or(&["color", "fgcolor"])
                .unwrap_or("")
                .split(','),
            bgcolor: scanner.next_or(&["back", "bgcolor"]),
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AfkArgs<'a> {
    pub challenge: Option<&'a str>,
}

impl<'a> From<&'a Arguments> for AfkArgs<'a> {
    fn from(args: &'a Arguments) -> Self {
        let mut scanner = args.scan();
        Self {
            challenge: scanner.next_or(&["challenge"]),
        }
    }
}
