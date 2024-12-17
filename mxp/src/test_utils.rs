use std::fmt::Display;
use std::str::FromStr;

pub type StringPairs<T, const N: usize> = [(T, &'static str); N];

pub fn format_from_pairs<T: Display, const N: usize>(
    pairs: &StringPairs<T, N>,
) -> ([String; N], [String; N]) {
    let mut actual = [const { String::new() }; N];
    let mut expected = [const { String::new() }; N];
    let iter = pairs.iter().zip(&mut actual).zip(&mut expected);
    for (((item, s), actual), expected) in iter {
        *actual = item.to_string();
        *expected = (*s).to_owned();
    }
    (actual, expected)
}

type ParseResult<T> = Result<T, <T as FromStr>::Err>;

pub fn parse_from_pairs<T: FromStr + Clone, const N: usize>(
    pairs: &StringPairs<T, N>,
) -> ([ParseResult<T>; N], [ParseResult<T>; N]) {
    let mut actual: [Option<ParseResult<T>>; N] = [const { None }; N];
    let mut expected: [Option<ParseResult<T>>; N] = [const { None }; N];
    let iter = pairs.iter().zip(&mut actual).zip(&mut expected);
    for (((item, s), actual), expected) in iter {
        *actual = Some(s.parse());
        *expected = Some(Ok(item.clone()));
    }
    (actual.map(Option::unwrap), expected.map(Option::unwrap))
}
