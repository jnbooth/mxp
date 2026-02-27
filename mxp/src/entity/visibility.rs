use flagset::FlagSet;

use crate::EntityKeyword;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum EntityVisibility {
    /// By default, MXP entities can be queried, but are not listed.
    #[default]
    Default,
    /// Private entities cannot be queried by the MUD client.  They are completely hidden.
    Private,
    /// Published entities can be used by the client to produce a list of MUD Server variables to be access by the player.
    Published,
}

impl From<FlagSet<EntityKeyword>> for EntityVisibility {
    fn from(value: FlagSet<EntityKeyword>) -> Self {
        if value.contains(EntityKeyword::Private) {
            Self::Private
        } else if value.contains(EntityKeyword::Publish) {
            Self::Published
        } else {
            Self::Default
        }
    }
}
