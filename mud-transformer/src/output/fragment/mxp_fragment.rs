use super::OutputFragment;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MxpFragment {
    Entity(EntityFragment),
    Error(mxp::Error),
    ExpireLinks(Option<String>),
    FileFilter(mxp::Filter),
    Gauge(mxp::Gauge),
    Music(mxp::Music),
    MusicOff,
    Relocate(mxp::Relocate),
    Sound(mxp::Sound),
    SoundOff,
    StatusBar(mxp::Stat),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EntityFragment {
    Set {
        name: String,
        value: String,
        publish: bool,
        is_variable: bool,
    },
    Unset {
        name: String,
        is_variable: bool,
    },
}

impl EntityFragment {
    pub fn entity(entry: &mxp::EntityEntry) -> Self {
        Self::new(entry, false)
    }

    pub fn variable(entry: &mxp::EntityEntry) -> Self {
        Self::new(entry, true)
    }

    fn new(entry: &mxp::EntityEntry, is_variable: bool) -> Self {
        match entry.value {
            Some(entity) => Self::Set {
                name: entry.name.to_owned(),
                value: entity.value.clone(),
                publish: entity.published,
                is_variable,
            },
            None => Self::Unset {
                name: entry.name.to_owned(),
                is_variable,
            },
        }
    }
}

impl From<EntityFragment> for OutputFragment {
    fn from(value: EntityFragment) -> Self {
        Self::Mxp(MxpFragment::Entity(value))
    }
}

impl From<MxpFragment> for OutputFragment {
    fn from(value: MxpFragment) -> Self {
        Self::Mxp(value)
    }
}

impl From<mxp::Error> for OutputFragment {
    fn from(value: mxp::Error) -> Self {
        Self::Mxp(MxpFragment::Error(value))
    }
}

impl From<mxp::Filter> for OutputFragment {
    fn from(value: mxp::Filter) -> Self {
        Self::Mxp(MxpFragment::FileFilter(value))
    }
}

impl From<mxp::Frame> for OutputFragment {
    fn from(value: mxp::Frame) -> Self {
        Self::Frame(value)
    }
}

impl From<mxp::Gauge> for OutputFragment {
    fn from(value: mxp::Gauge) -> Self {
        Self::Mxp(MxpFragment::Gauge(value))
    }
}

impl From<mxp::Image> for OutputFragment {
    fn from(value: mxp::Image) -> Self {
        Self::Image(value)
    }
}

impl From<mxp::Music> for OutputFragment {
    fn from(value: mxp::Music) -> Self {
        Self::Mxp(MxpFragment::Music(value))
    }
}

impl From<mxp::Relocate> for OutputFragment {
    fn from(value: mxp::Relocate) -> Self {
        Self::Mxp(MxpFragment::Relocate(value))
    }
}

impl From<mxp::Sound> for OutputFragment {
    fn from(value: mxp::Sound) -> Self {
        Self::Mxp(MxpFragment::Sound(value))
    }
}

impl From<mxp::Stat> for OutputFragment {
    fn from(value: mxp::Stat) -> Self {
        Self::Mxp(MxpFragment::StatusBar(value))
    }
}
