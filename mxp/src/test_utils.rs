use std::fmt::Display;
use std::str::FromStr;

pub type StringPair<T> = (T, &'static str);

macro_rules! const_nonzero {
    ($n:expr) => {{
        const _: () = ::std::assert!($n != 0);

        match ::std::num::NonZero::new($n) {
            Some(n) => n,
            None => unreachable!(),
        }
    }};
}
pub(crate) use const_nonzero;

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
