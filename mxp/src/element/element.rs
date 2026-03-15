use std::num::NonZero;

use super::decoder::DecodeElement;
use super::item::ElementItem;
use super::parse_as::ParseAs;
use super::tag::Tag;
use crate::parse::{Arguments, Decoder, Words};

/// User-defined MXP tags that we recognise, e.g. <boldcolor>.
/// For example: <!ELEMENT boldtext '<COLOR &col;><B>' ATT='col=red'>
///
/// See [MXP specification: Elements](https://www.zuggsoft.com/zmud/mxp.htm#ELEMENT).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Element {
    /// Tag name
    pub name: String,
    /// What atomic elements it defines (arg 1)
    pub items: Vec<ElementItem>,
    /// List of attributes to this element (ATT="xx")
    pub attributes: Arguments<'static, String>,
    /// Line tag number (20 - 99) (TAG=n)
    pub tag: Option<NonZero<u8>>,
    /// Parsing flag
    pub parse_as: Option<ParseAs>,
    /// Which variable to set (SET x)
    pub variable: Option<String>,
    /// Whether the element is open (OPEN)
    pub open: bool,
    /// Whether the element has no closing tag (EMPTY)
    pub command: bool,
}

impl Element {
    pub fn decode<'a, D>(&'a self, args: &'a Arguments<'a>, decoder: D) -> DecodeElement<'a, D>
    where
        D: Decoder + Copy,
    {
        DecodeElement::new(self, args, decoder)
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
