use std::fmt;

use super::atomic_tag::AtomicTag;
use super::decoder::ElementDecoder;
use super::item::ElementItem;
use super::parse_as::ParseAs;
use crate::LineTagProperties;
use crate::arguments::Arguments;
use crate::line::Mode;
use crate::parse::{Decoder, Words};

/// User-defined MXP tags.
///
/// An `Element` is a combination of [`AtomicTag`]s, with its own argument schema. When a custom
/// element is used, the arguments supplied to it are parsed and passed on to its child
/// [`items`], each of which applies an [`Action`].
///
/// An element is defined by an [`ElementDefinition`] in a [`Definition`].
///
/// See [MXP specification: Elements](https://www.zuggsoft.com/zmud/mxp.htm#ELEMENT).
///
/// [`items`]: Self::items
/// [`Action`]: crate::Action
/// [`ElementDefinition`]: crate::node::ElementDefinition
/// [`Definition`]: crate::node::Definition
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
    /// OPEN elements can be used in any [`Mode`]. By default, elements can only be used if the
    /// current line mode is not [OPEN](Mode::is_open).
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

    /// Retrieves the additional line tag properties defined for this element if [`line_tag`]
    /// is Some.
    ///
    /// [`line_tag`]: Self::line_tag
    pub fn line_tag_properties<'a>(
        &self,
        state: &'a crate::State,
    ) -> Option<&'a LineTagProperties> {
        Some(state.get_line_tag(self.line_tag?)?.properties)
    }

    pub(crate) fn well_known() -> [(String, Element); 8] {
        const COLOR: &AtomicTag = AtomicTag::well_known("color").unwrap();

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

impl fmt::Display for Element {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Note: element definitions aren't decoded, so nothing here needs to be escaped.
        let Self {
            name,
            items,
            attributes,
            line_tag,
            parse_as,
            variable,
            open,
            command,
        } = self;
        write!(f, "<!EL {name}'")?;
        for item in items {
            write!(f, "{item}")?;
        }
        f.write_str("'")?;
        if !attributes.is_empty() {
            write!(f, " ATT='{attributes}'")?;
        }
        if let Some(line_tag) = line_tag {
            write!(f, " TAG={line_tag}")?;
        }
        if let Some(parse_as) = parse_as {
            write!(f, " FLAG=\"{parse_as}\"")?;
        }
        if let Some(variable) = variable {
            write!(f, " FLAG=\"SET {variable}\"")?;
        }
        if *open {
            f.write_str(" OPEN")?;
        }
        if *command {
            f.write_str(" EMPTY")?;
        }
        f.write_str(">")
    }
}
