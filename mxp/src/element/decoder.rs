use std::borrow::Cow;
use std::iter::FusedIterator;
use std::slice;

use super::action::Action;
use super::element::Element;
use super::item::ElementItem;
use crate::arguments::Arguments;
use crate::keyword::KeywordFilter;
use crate::parse::Decoder;

#[derive(Copy, Clone, Debug)]
struct DecodeElement<'a, D: Decoder> {
    decoder: D,
    attributes: &'a Arguments<'static, String>,
    args: &'a Arguments<'a>,
}

impl<D: Decoder> Decoder for DecodeElement<'_, D> {
    fn get_entity<K: KeywordFilter>(&self, name: &str) -> Option<&str> {
        match self.args.find_from_attributes::<K>(name, self.attributes) {
            Some(attr) => Some(attr),
            None => self.decoder.get_entity::<K>(name),
        }
    }
}

/// This struct is created by [`State::decode_element`](crate::State::decode_element).
/// See its documentation for more.
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Clone)]
pub struct ElementDecoder<'a, D: Decoder + Copy> {
    decoder: DecodeElement<'a, D>,
    items: slice::Iter<'a, ElementItem>,
}

impl<'a, D: Decoder + Copy> ElementDecoder<'a, D> {
    pub(super) fn new(element: &'a Element, args: &'a Arguments<&'a str>, decoder: D) -> Self {
        Self {
            decoder: DecodeElement {
                decoder,
                attributes: &element.attributes,
                args,
            },
            items: element.items.iter(),
        }
    }
}

impl<'a, D: Decoder + Copy> Iterator for ElementDecoder<'a, D> {
    type Item = crate::Result<Action<Cow<'a, str>>>;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.items.next()?;
        let scanner = item.arguments.scan(self.decoder);
        Some(
            Action::decode(item.tag.action, scanner)
                .map_err(|e| e.with_context(format_args!(" for <{}>", item.tag.name))),
        )
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let exact = self.len();
        (exact, Some(exact))
    }
}

impl<D> ExactSizeIterator for ElementDecoder<'_, D>
where
    D: Decoder + Copy,
{
    fn len(&self) -> usize {
        self.items.len()
    }
}

impl<D> FusedIterator for ElementDecoder<'_, D> where D: Decoder + Copy {}

#[cfg(test)]
mod tests {
    use crate::element::Action;
    use crate::elements::Color;
    use crate::test_utils::{decode_actions, try_from_node};

    fn foreground(name: &str) -> Color {
        Color {
            fore: Some(crate::RgbColor::named(name).unwrap()),
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
    }
}
