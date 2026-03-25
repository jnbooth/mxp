use std::borrow::Cow;
use std::iter::FusedIterator;
use std::slice;

use super::action::Action;
use super::element::Element;
use super::item::ElementItem;
use crate::arguments::Arguments;
use crate::element::AttributeList;
use crate::entity::DecodedEntity;
use crate::parse::Decoder;

/// This struct is created by [`Element::decoder`](crate::element::Element::decoder).
/// See its documentation for more.
#[derive(Copy, Clone, Debug)]
pub struct ElementDecoder<'a, D: Decoder> {
    decoder: D,
    attributes: &'a AttributeList,
    args: &'a Arguments<'a>,
}

impl<'a, D: Decoder> ElementDecoder<'a, D> {
    pub(super) fn new(decoder: D, attributes: &'a AttributeList, args: &'a Arguments<'a>) -> Self {
        Self {
            decoder,
            attributes,
            args,
        }
    }
}

impl<'a, D: Decoder> ElementDecoder<'a, D> {
    fn find(&self, name: &str) -> Option<&'a str> {
        self.attributes.find(name, self.args)
    }
}

impl<D: Decoder> Decoder for ElementDecoder<'_, D> {
    fn get_entity(&self, name: &str) -> Option<&str> {
        self.find(name).or_else(|| self.decoder.get_entity(name))
    }

    fn decode_entity(&self, name: &str) -> crate::Result<DecodedEntity<'_>> {
        match self.find(name) {
            Some(attr) => Ok(self.decoder.decode_string(attr)?.into()),
            None => self.decoder.decode_entity(name),
        }
    }
}

/// This struct is created by [`Element::decode`](crate::element::Element::decode).
/// See its documentation for more.
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Clone)]
pub struct ElementDecodeIter<'a, D: Decoder + Copy> {
    decoder: ElementDecoder<'a, D>,
    items: slice::Iter<'a, ElementItem>,
}

impl<'a, D: Decoder + Copy> ElementDecodeIter<'a, D> {
    pub(super) fn new(element: &'a Element, args: &'a Arguments<&'a str>, decoder: D) -> Self {
        Self {
            decoder: ElementDecoder {
                decoder,
                attributes: &element.attributes,
                args,
            },
            items: element.items.iter(),
        }
    }
}

impl<'a, D: Decoder + Copy> Iterator for ElementDecodeIter<'a, D> {
    type Item = crate::Result<Action<Cow<'a, str>>>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.items.next()?.decode(self.decoder))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let exact = self.len();
        (exact, Some(exact))
    }
}

impl<D: Decoder + Copy> ExactSizeIterator for ElementDecodeIter<'_, D> {
    #[inline]
    fn len(&self) -> usize {
        self.items.len()
    }
}

impl<D: Decoder + Copy> FusedIterator for ElementDecodeIter<'_, D> {}

#[cfg(test)]
mod tests {
    use crate::color::RgbColor;
    use crate::element::Action;
    use crate::elements::Color;
    use crate::test_utils::{decode_actions, try_from_node};

    fn color(name: &str) -> RgbColor {
        RgbColor::named(name).unwrap()
    }

    fn foreground(name: &str) -> Color {
        Color {
            fore: Some(color(name)),
            ..Default::default()
        }
    }

    #[test]
    fn custom_args() {
        let mut state = crate::State::default();
        let definition = try_from_node("<!ELEMENT mycolor '<COLOR &col;>' ATT='col=red'>");
        state.define(definition).unwrap();
        let red = decode_actions("<mycolor>", &state).unwrap();
        assert_eq!(red, &[Action::Color(foreground("red"))]);
        let blue = decode_actions("<mycolor col=blue>", &state).unwrap();
        assert_eq!(blue, &[Action::Color(foreground("blue"))]);
        let reset = decode_actions("<mycolor col=\"\">", &state).unwrap();
        assert_eq!(reset, &[Action::Color(Color::default())]);

        let definition = try_from_node("<!ELEMENT mycolors '<COLOR &col; &bg;>' ATT='bg col=red'>");
        state.define(definition).unwrap();
        let positional = decode_actions("<mycolors col=blue green>", &state).unwrap();
        assert_eq!(
            positional,
            &[Action::Color(Color {
                fore: Some(color("blue")),
                back: Some(color("green")),
            })]
        );
    }
}
