use std::iter::Filter;
use std::slice;
use std::str::FromStr;

pub trait KeywordFilter {
    type Iter<'a>: Iterator<Item = &'a String>;

    fn iter(args: &[String]) -> Self::Iter<'_>;
}

pub struct NoKeywords;

impl KeywordFilter for NoKeywords {
    type Iter<'a> = slice::Iter<'a, String>;

    fn iter(args: &[String]) -> Self::Iter<'_> {
        args.iter()
    }
}

impl<K: FromStr> KeywordFilter for K {
    type Iter<'a> = Filter<slice::Iter<'a, String>, fn(&&String) -> bool>;

    fn iter(args: &[String]) -> Self::Iter<'_> {
        args.iter().filter(|arg| K::from_str(arg).is_err())
    }
}
