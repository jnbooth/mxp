use super::OutputFragment;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ControlFragment {
    Backspace,
    Beep,
    EraseCharacter,
    EraseLine,
}

impl ControlFragment {
    pub(super) const fn should_flush(&self) -> bool {
        !matches!(self, Self::Beep)
    }
}

impl From<ControlFragment> for OutputFragment {
    fn from(value: ControlFragment) -> Self {
        Self::Control(value)
    }
}
