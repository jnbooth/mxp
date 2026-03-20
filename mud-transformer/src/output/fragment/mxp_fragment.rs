use bytestring::ByteString;

use super::OutputFragment;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MxpFragment {
    Entity(EntityFragment),
    Error(mxp::Error),
    Expire(mxp::Expire),
    Filter(mxp::Filter),
    Frame(mxp::Frame),
    Gauge(mxp::Gauge),
    Mapper(MapperFragment),
    Music(mxp::Music),
    MusicOff,
    Relocate(mxp::Relocate),
    Sound(mxp::Sound),
    SoundOff,
    Stat(mxp::Stat),
    StyleVersion(mxp::StyleVersion),
    Variable(VariableFragment),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityFragment {
    pub name: ByteString,
    pub value: Option<ByteString>,
    pub publish: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MapperFragment {
    pub parse_as: mxp::ParseAs,
    pub value: ByteString,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VariableFragment {
    pub name: ByteString,
    pub value: ByteString,
}

impl From<MxpFragment> for OutputFragment {
    fn from(value: MxpFragment) -> Self {
        Self::Mxp(value)
    }
}

impl From<EntityFragment> for OutputFragment {
    fn from(value: EntityFragment) -> Self {
        Self::Mxp(MxpFragment::Entity(value))
    }
}

impl From<mxp::Error> for OutputFragment {
    fn from(value: mxp::Error) -> Self {
        Self::Mxp(MxpFragment::Error(value))
    }
}

impl From<mxp::Expire> for OutputFragment {
    fn from(value: mxp::Expire) -> Self {
        Self::Mxp(MxpFragment::Expire(value))
    }
}

impl From<mxp::Filter> for OutputFragment {
    fn from(value: mxp::Filter) -> Self {
        Self::Mxp(MxpFragment::Filter(value))
    }
}

impl From<mxp::Frame> for OutputFragment {
    fn from(value: mxp::Frame) -> Self {
        Self::Mxp(MxpFragment::Frame(value))
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

impl From<MapperFragment> for OutputFragment {
    fn from(value: MapperFragment) -> Self {
        Self::Mxp(MxpFragment::Mapper(value))
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
        Self::Mxp(MxpFragment::Stat(value))
    }
}

impl From<mxp::StyleVersion> for OutputFragment {
    fn from(value: mxp::StyleVersion) -> Self {
        Self::Mxp(MxpFragment::StyleVersion(value))
    }
}

impl From<VariableFragment> for OutputFragment {
    fn from(value: VariableFragment) -> Self {
        Self::Mxp(MxpFragment::Variable(value))
    }
}
