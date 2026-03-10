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
