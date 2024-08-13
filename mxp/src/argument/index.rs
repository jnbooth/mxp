/// MXP elements can have both positional and named arguments.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ArgumentIndex<'a> {
    Positional(usize),
    Named(&'a str),
}
impl<'a> From<usize> for ArgumentIndex<'a> {
    fn from(value: usize) -> Self {
        Self::Positional(value)
    }
}
impl<'a> From<&'a str> for ArgumentIndex<'a> {
    fn from(value: &'a str) -> Self {
        Self::Named(value)
    }
}
impl<'a> ArgumentIndex<'a> {
    pub fn is_positional(self) -> bool {
        match self {
            Self::Positional(_) => true,
            Self::Named(_) => false,
        }
    }

    pub fn is_named(self) -> bool {
        match self {
            Self::Positional(_) => false,
            Self::Named(_) => true,
        }
    }
}
