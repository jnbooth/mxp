use super::decoder::ElementDecoder;
use super::item::ElementItem;
use super::parse_as::ParseAs;
use super::tag::Tag;
use crate::arguments::Arguments;
use crate::line::Mode;
use crate::parse::{Decoder, Words};

/// User-defined MXP tags.
///
/// An `Element` is a combination of atomic [`Tag`]s, with its own argument schema. When a custom
/// element is used, the arguments supplied to it are parsed and passed on to its child
/// [`items`](Self::items), each of which applies an [`Action`](crate::Action).
///
/// An element is defined by an [`ElementDefinition`](crate::parsed::ElementDefinition) in a
/// [`ParsedDefinition`](crate::parsed::ParsedDefinition).
///
/// See [MXP specification: Elements](https://www.zuggsoft.com/zmud/mxp.htm#ELEMENT).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Element {
    /// Tag name.
    pub name: String,
    /// Atomic tags declared in its definition string.
    pub items: Vec<ElementItem>,
    /// Arguments or attributes declared in the element's definition or in a later `<!ATTLIST>`.
    pub attributes: Arguments<'static, String>,
    /// Line tag mode, if the element is associated with a user-defined line tag.
    pub line_tag: Option<Mode>,
    /// If specified, text contained by this element should be parsed in a specific way by an
    /// automapper.
    pub parse_as: Option<ParseAs>,
    /// If specified, text contained by this element should be stored as a local variable with the
    /// given name.
    pub variable: Option<String>,
    /// Open elements can be used in any [`Mode`]. By default, elements are secure (not open),
    /// meaning they can only be used if the current line mode is [secure](Mode::is_secure).
    pub open: bool,
    /// Command tags do not have content, so they have no closing tag.
    pub command: bool,
}

impl Element {
    pub fn decode<'a, D>(&'a self, args: &'a Arguments<'a>, decoder: D) -> ElementDecoder<'a, D>
    where
        D: Decoder + Copy,
    {
        ElementDecoder::new(self, args, decoder)
    }

    pub(crate) fn well_known() -> [(String, Element); 8] {
        const COLOR: &Tag = Tag::well_known("color").unwrap();

        fn color_el(name: &str, body: &str) -> (String, Element) {
            (
                name.to_owned(),
                Element {
                    name: name.to_owned(),
                    open: true,
                    items: vec![ElementItem {
                        tag: COLOR,
                        arguments: Words::new(body).try_into().unwrap(),
                    }],
                    ..Default::default()
                },
            )
        }

        [
            color_el("BlackMXP", "fore=#000000"),
            color_el("RedMXP", "fore=#FF0000"),
            color_el("GreenMXP", "fore=#008000"),
            color_el("YellowMXP", "fore=#FFFF00"),
            color_el("BlueMXP", "fore=#0000FF"),
            color_el("MagentaMXP", "fore=#FF00FF"),
            color_el("CyanMXP", "fore=#00FFFF"),
            color_el("WhiteMXP", "fore=#FFFFFF"),
        ]
    }
}
