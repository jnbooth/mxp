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

#[cfg(test)]
mod tests {
    use crate::keyword::ElementKeyword;

    use super::*;

    #[test]
    fn filter_keywords() {
        let args = ["thing1", "open", "thing2", "empty", "empty", "thing3"]
            .into_iter()
            .map(ToOwned::to_owned)
            .collect::<Vec<String>>();
        let non_keywords = <ElementKeyword as KeywordFilter>::iter(&args)
            .map(String::as_str)
            .collect::<Vec<&str>>();
        assert_eq!(non_keywords, vec!["thing1", "thing2", "thing3"]);
    }
}
