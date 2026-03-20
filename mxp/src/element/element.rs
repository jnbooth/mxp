use std::fmt;

use super::atomic_tag::AtomicTag;
use super::decoder::{ElementDecodeIter, ElementDecoder};
use super::flag::ElementFlag;
use super::item::ElementItem;
use super::parse_as::ParseAs;
use crate::LineTagProperties;
use crate::arguments::Arguments;
use crate::element::AttributeList;
use crate::line::Mode;
use crate::parse::Decoder;

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
    pub attributes: AttributeList,
    /// Line tag mode, if the element is associated with a user-defined line tag.
    pub line_tag: Option<Mode>,
    /// See [`ElementFlag`].
    pub flag: Option<ElementFlag>,
    /// OPEN elements can be used in any [`Mode`]. By default, elements can only be used if the
    /// current line mode is not [OPEN](Mode::is_open).
    pub open: bool,
    /// Command tags do not have content, so they have no closing tag.
    pub empty: bool,
}

impl Element {
    /// If [`self.flag`] is [`Some(ElementFlag::ParseAs(parse_as))`], returns `parse_as`.
    ///
    /// [`self.flag`]: Element::flag
    ///[`Some(ElementFlag::ParseAs(parse_as))`]: ElementFlag::ParseAs
    pub fn parse_as(&self) -> Option<ParseAs> {
        match self.flag {
            Some(ElementFlag::ParseAs(parse_as)) => Some(parse_as),
            _ => None,
        }
    }

    /// If [`self.flag`] is [`Some(ElementFlag::Set(variable))`], returns `variable`.
    ///
    /// [`self.flag`]: Element::flag
    ///[`Some(ElementFlag::Set(variable))`]: ElementFlag::Set
    pub fn variable(&self) -> Option<&str> {
        match &self.flag {
            Some(ElementFlag::Set(variable)) => Some(variable),
            _ => None,
        }
    }

    /// Returns an iterator that decodes all child actions from [`items`](Self::items).
    /// The iterator element type is `mxp::Result<Action<Cow<'a, str>>>`.
    /// See also [`Action`](crate::Action).
    pub fn decode<'a, D>(&'a self, args: &'a Arguments<'a>, decoder: D) -> ElementDecodeIter<'a, D>
    where
        D: Decoder + Copy,
    {
        ElementDecodeIter::new(self, args, decoder)
    }

    /// Returns a decoder used with [`ElementItem::decode`].
    pub fn decoder<'a, D>(&'a self, args: &'a Arguments<'a>, decoder: D) -> ElementDecoder<'a, D>
    where
        D: Decoder + Copy,
    {
        ElementDecoder::new(decoder, &self.attributes, args)
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

        fn color_el(name: &str, fore: &str) -> (String, Element) {
            let mut arguments = Arguments::new();
            arguments.insert("fore", fore.to_owned());
            (
                name.to_owned(),
                Element {
                    name: name.to_owned(),
                    open: true,
                    items: vec![ElementItem {
                        tag: COLOR,
                        arguments,
                    }],
                    ..Default::default()
                },
            )
        }

        [
            color_el("BlackMXP", "#000000"),
            color_el("RedMXP", "#FF0000"),
            color_el("GreenMXP", "#008000"),
            color_el("YellowMXP", "#FFFF00"),
            color_el("BlueMXP", "#0000FF"),
            color_el("MagentaMXP", "#FF00FF"),
            color_el("CyanMXP", "#00FFFF"),
            color_el("WhiteMXP", "#FFFFFF"),
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
            flag,
            open,
            empty: command,
        } = self;
        write!(f, "<!EL {name} '")?;
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
        if let Some(flag) = flag {
            write!(f, " FLAG=\"{flag}\"")?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fmt() {
        let element = Element {
            name: "custom".to_owned(),
            items: ElementItem::parse_all("<COLOR &col;><B>").unwrap(),
            attributes: "col=red".parse().unwrap(),
            line_tag: Some(Mode(30)),
            flag: Some(ElementFlag::Set("myvar".to_owned())),
            open: true,
            empty: true,
        };
        assert_eq!(
            element.to_string(),
            "<!EL custom '<COLOR &col;><B>' ATT='col=red' TAG=30 FLAG=\"SET myvar\" OPEN EMPTY>"
        );
    }
}
