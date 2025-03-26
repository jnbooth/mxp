use std::iter::Filter;
use std::slice;
use std::str::FromStr;

/// A trait for filtering out keywords from a list of strings.
///
/// `KeywordFilter` is implemented for all types that implement [`FromStr`], filtering out strings
/// that can be parsed to the type.
pub trait KeywordFilter {
    /// The type of iterator produced by the filter.
    type Iter<'a, S: 'a>: Iterator<Item = &'a S>;

    /// Filters keywords out from a list of strings.
    fn iter<'a, S: AsRef<str> + 'a>(args: &'a [S]) -> Self::Iter<'a, S>;
}

/// Default [`KeywordFilter`] that iterates through strings without filtering them.
pub struct NoKeywords;

impl KeywordFilter for NoKeywords {
    type Iter<'a, S: 'a> = slice::Iter<'a, S>;

    fn iter<'a, S: AsRef<str> + 'a>(args: &'a [S]) -> Self::Iter<'a, S> {
        args.iter()
    }
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_not_keyword<K: FromStr, S: AsRef<str>>(arg: &&S) -> bool {
    K::from_str(arg.as_ref()).is_err()
}

impl<K: FromStr> KeywordFilter for K {
    type Iter<'a, S: 'a> = Filter<slice::Iter<'a, S>, fn(&&S) -> bool>;

    fn iter<'a, S: AsRef<str> + 'a>(args: &'a [S]) -> Self::Iter<'a, S> {
        args.iter().filter(is_not_keyword::<K, S>)
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
