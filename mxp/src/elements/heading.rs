use std::str::FromStr;

/// Sets text heading level.
///
/// See [MXP Specification: HTML tags](https://www.zuggsoft.com/zmud/mxp.htm#HTML%20tags).
///
/// # Examples
///
/// ```
/// assert_eq!("<H1>".parse::<mxp::Heading>(), Ok(mxp::Heading::H1));
/// assert_eq!("<H2>".parse::<mxp::Heading>(), Ok(mxp::Heading::H2));
/// assert_eq!("<H3>".parse::<mxp::Heading>(), Ok(mxp::Heading::H3));
/// assert_eq!("<H4>".parse::<mxp::Heading>(), Ok(mxp::Heading::H4));
/// assert_eq!("<H5>".parse::<mxp::Heading>(), Ok(mxp::Heading::H5));
/// assert_eq!("<H6>".parse::<mxp::Heading>(), Ok(mxp::Heading::H6));
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Heading {
    H1 = 1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

impl Heading {
    /// # Examples
    ///
    /// ```
    /// assert_eq!(mxp::Heading::H1.level(), 1);
    /// assert_eq!(mxp::Heading::H5.level(), 5);
    /// ```
    pub const fn level(self) -> u8 {
        self as u8
    }
}

impl FromStr for Heading {
    type Err = crate::parse::FromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = crate::parse::cleanup_source(s)?;
        let [b'h' | b'H', code] = s.as_bytes() else {
            return Err(Self::Err::UnexpectedTag(s.to_owned()));
        };
        match code {
            b'1' => Ok(Self::H1),
            b'2' => Ok(Self::H2),
            b'3' => Ok(Self::H3),
            b'4' => Ok(Self::H4),
            b'5' => Ok(Self::H5),
            b'6' => Ok(Self::H6),
            _ => Err(Self::Err::UnexpectedTag(s.to_owned())),
        }
    }
}
