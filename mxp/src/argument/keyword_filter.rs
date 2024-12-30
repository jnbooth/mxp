use std::iter::Filter;
use std::slice;
use std::str::FromStr;

pub trait KeywordFilter {
    type Iter<'a, S: 'a>: Iterator<Item = &'a S>;

    fn iter<'a, S: AsRef<str> + 'a>(args: &'a [S]) -> Self::Iter<'a, S>;
}

pub struct NoKeywords;

impl KeywordFilter for NoKeywords {
    type Iter<'a, S: 'a> = slice::Iter<'a, S>;

    fn iter<'a, S: AsRef<str> + 'a>(args: &'a [S]) -> Self::Iter<'a, S> {
        args.iter()
    }
}

impl<K: FromStr> KeywordFilter for K {
    type Iter<'a, S: 'a> = Filter<slice::Iter<'a, S>, fn(&&S) -> bool>;

    fn iter<'a, S: AsRef<str> + 'a>(args: &'a [S]) -> Self::Iter<'a, S> {
        let filter: fn(&&S) -> bool = |s| K::from_str(s.as_ref()).is_err();
        args.iter().filter(filter)
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
            .collect::<Vec<&str>>();
        let non_keywords = <ElementKeyword as KeywordFilter>::iter(&args)
            .copied()
            .collect::<Vec<&str>>();
        assert_eq!(non_keywords, vec!["thing1", "thing2", "thing3"]);
    }
}
