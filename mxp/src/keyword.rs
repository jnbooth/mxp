use flagset::flags;

use crate::parse::UnrecognizedVariant;

flags! {
    /// Keywords for [`<DEST>`](crate::DEST) tags.
    pub(crate) enum DestKeyword: u8 {
        /// Causes the rest of the frame to be erased after displaying the text.
        Eof,
        /// Causes the rest of the line to be erased after displaying the text.
        Eol,
    }

    /// Keywords for [`<!ELEMENT>`](crate::Element) tags.
    pub(crate) enum ElementKeyword: u8 {
        Open,
        Empty,
        Delete,
    }

    /// Keywords for [`<!ENTITY>`](crate::Entity) tags.
    pub enum EntityKeyword: u8 {
        /// PRIVATE entities cannot be queried by the MUD client. They are completely hidden.
        Private,
        /// PUBLISH entities can be used by the client to produce a list of MUD Server variables to
        /// be accessed by the player.
        Publish,
        /// To delete an entity, use the DELETE argument. Setting an entity to a empty value does
        /// not delete it.
        Delete,
        /// The ADD argument causes the Value to be added as a new item in a string list. So, it is
        /// appended to the existing value of the variable. String lists are values separated by the
        /// `'|'` character.
        Add,
        /// The REMOVE argument causes the Value to be removed from the existing string list.
        Remove,
    }

    /// Keywords for [`<FRAME>`](crate::Frame) tags.
    pub(crate) enum FrameKeyword: u8 {
        Floating,
        Internal,
    }

    /// Keywords for [`<IMAGE>`](crate::Image) tags.
    pub(crate) enum ImageKeyword: u8 {
        IsMap,
    }

    /// Keywords for line tag updates.
    pub(crate) enum LineTagKeyword: u8 {
        Gag,
        Enable,
        Disable,
    }

    /// Keywords for [`<RELOCATE>`](crate::Relocate) tags.
    pub(crate) enum RelocateKeyword: u8 {
        Quiet,
    }

    /// Keywords for [`<SEND>`](crate::Link) tags.
    pub(crate) enum SendKeyword: u8 {
        Prompt,
    }
}

impl_parse_enum!(DestKeyword, Eof, Eol);

impl_parse_enum!(ElementKeyword, Open, Empty, Delete);

impl_parse_enum!(EntityKeyword, Private, Publish, Delete, Add, Remove);
impl_display_enum!(EntityKeyword, Private, Publish, Delete, Add, Remove);

impl_parse_enum!(FrameKeyword, Floating, Internal);

impl_parse_enum!(ImageKeyword, IsMap);

impl_parse_enum!(RelocateKeyword, Quiet);

impl_parse_enum!(SendKeyword, Prompt);

impl_parse_enum!(LineTagKeyword, Gag, Enable, Disable);
