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
struct ElementDecoder<'a, D: Decoder> {
    decoder: D,
    element: &'a Element,
    args: &'a Arguments<'a>,
}

impl<D: Decoder> Decoder for ElementDecoder<'_, D> {
    fn get_entity<K: KeywordFilter>(&self, name: &str) -> Option<&str> {
        match self
            .args
            .find_from_attributes::<K>(name, &self.element.attributes)
        {
            Some(attr) => Some(attr),
            None => self.decoder.get_entity::<K>(name),
        }
    }
}

/// This `struct` is created by [`State::decode_element`]. See its documentation for more.
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Clone)]
pub struct DecodeElement<'a, D: Decoder + Copy> {
    decoder: ElementDecoder<'a, D>,
    items: slice::Iter<'a, ElementItem>,
}

impl<'a, D: Decoder + Copy> DecodeElement<'a, D> {
    pub(super) fn new(element: &'a Element, args: &'a Arguments<&'a str>, decoder: D) -> Self {
        Self {
            decoder: ElementDecoder {
                decoder,
                element,
                args,
            },
            items: element.items.iter(),
        }
    }
}

impl<'a, D: Decoder + Copy> Iterator for DecodeElement<'a, D> {
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

impl<D> ExactSizeIterator for DecodeElement<'_, D>
where
    D: Decoder + Copy,
{
    fn len(&self) -> usize {
        self.items.len()
    }
}

impl<D> FusedIterator for DecodeElement<'_, D> where D: Decoder + Copy {}
