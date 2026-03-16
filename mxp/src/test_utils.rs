use std::fmt::Display;
use std::str::FromStr;

use crate::element::Action;
use crate::parsed::{ParsedDefinition, ParsedElement, ParsedTagOpen};
use crate::{Component, State};

pub type StringPair<T> = (T, &'static str);

fn strip_brackets(mut source: &str) -> &str {
    source = source.strip_prefix('<').unwrap_or(source);
    source.strip_suffix('>').unwrap_or(source)
}

#[track_caller]
pub fn parse_definition(source: &str) -> ParsedDefinition<'_> {
    match ParsedElement::parse(strip_brackets(source), true) {
        Ok(ParsedElement::Definition(definition)) => definition,
        Ok(ParsedElement::TagClose(_)) => panic!("expected definition, got closing tag"),
        Ok(ParsedElement::TagOpen(_)) => panic!("expected definition, got opening tag"),
        Err(e) => panic!("failed to parse definition: {e}"),
    }
}

#[track_caller]
pub fn parse_tag_open(source: &str) -> ParsedTagOpen<'_> {
    match ParsedElement::parse(strip_brackets(source), true) {
        Ok(ParsedElement::Definition(_)) => panic!("expected opening tag, got definition"),
        Ok(ParsedElement::TagClose(_)) => panic!("expected opening tag, got closing tag"),
        Ok(ParsedElement::TagOpen(tag)) => tag,
        Err(e) => panic!("failed to parse opening tag: {e}"),
    }
}

#[track_caller]
pub fn decode_actions(source: &str, state: &State) -> crate::Result<Vec<Action>> {
    let tag = parse_tag_open(source);
    match state.get_component(tag.name, true)? {
        Component::AtomicTag(atom) => Ok(vec![atom.decode(&tag.arguments, state)?.into_owned()]),
        Component::Element(el) => el
            .decode(&tag.arguments, state)
            .map(|result| result.map(Action::into_owned))
            .collect(),
    }
}

fn zip_pairs<T, U, F>(pairs: &[StringPair<T>], mut zipper: F) -> (Vec<U>, Vec<U>)
where
    F: FnMut(&T, &'static str) -> (U, U),
{
    let mut actual = Vec::with_capacity(pairs.len());
    let mut expected = Vec::with_capacity(pairs.len());
    for (item, s) in pairs {
        let (actual_result, expected_result) = zipper(item, s);
        actual.push(actual_result);
        expected.push(expected_result);
    }
    (actual, expected)
}

pub fn format_from_pairs<T: Display>(pairs: &[StringPair<T>]) -> (Vec<String>, Vec<String>) {
    zip_pairs(pairs, |item, s| (item.to_string(), (*s).to_owned()))
}

type ParseResult<T> = Result<T, <T as FromStr>::Err>;

pub fn parse_from_pairs<T: FromStr + Clone>(
    pairs: &[StringPair<T>],
) -> (Vec<ParseResult<T>>, Vec<ParseResult<T>>) {
    zip_pairs(pairs, |item, s| (s.parse(), Ok(item.clone())))
}
