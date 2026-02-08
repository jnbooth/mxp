#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum LocatorReporting {
    #[default]
    Disabled = 0,
    Enabled,
    EnabledOnce,
}

impl LocatorReporting {
    pub(crate) const fn from_code(code: Option<u16>) -> Option<Self> {
        match code {
            None | Some(0) => Some(Self::Disabled),
            Some(1) => Some(Self::Enabled),
            Some(2) => Some(Self::EnabledOnce),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum LocatorUnit {
    Pixels = 1,
    #[default]
    Characters,
}

impl LocatorUnit {
    pub(crate) const fn from_code(code: Option<u16>) -> Option<Self> {
        match code {
            None | Some(0 | 2) => Some(Self::Characters),
            Some(1) => Some(Self::Pixels),
            _ => None,
        }
    }
}
