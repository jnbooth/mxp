use flagset::flags;

use crate::parser::UnrecognizedVariant;

flags! {
    /// Keywords for [`<DEST>`](crate::DEST) tags.
    pub enum DestKeyword: u8 {
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
        Private,
        Publish,
        Delete,
        Add,
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

    /// Keywords for [`<MXP>`](crate::MXP) tags.
    pub enum MxpKeyword: u8 {
        Off,
        DefaultLocked,
        DefaultSecure,
        DefaultOpen,
        IgnoreNewlines,
        UseNewlines,
    }

    /// Keywords for [`<SEND>`](crate::Link) tags.
    pub(crate) enum SendKeyword: u8 {
        Prompt
    }
}

impl_parse_enum!(DestKeyword, Eof, Eol);

impl_parse_enum!(ElementKeyword, Open, Empty, Delete);

impl_parse_enum!(EntityKeyword, Private, Publish, Delete, Add, Remove);

impl_parse_enum!(FrameKeyword, Floating, Internal);

impl_parse_enum!(ImageKeyword, IsMap);

impl_parse_enum!(
    MxpKeyword,
    Off,
    DefaultOpen,
    DefaultSecure,
    DefaultLocked,
    UseNewlines,
    IgnoreNewlines
);

impl_parse_enum!(SendKeyword, Prompt);

impl_parse_enum!(LineTagKeyword, Gag, Enable, Disable);
