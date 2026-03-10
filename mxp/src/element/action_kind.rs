use flagset::flags;

flags! {
    pub enum ActionKind: u64 {
        /// bold
        Bold,
        /// Hard Line break (secure)
        Br,
        /// eg. <color fore=red back=blue>
        Color,
        /// destination frame
        Dest,
        /// expire
        Expire,
        /// sound/image filter
        Filter,
        /// Font appearance
        Font,
        /// frame
        Frame,
        /// gauge
        Gauge,
        /// Level 1 heading (secure)
        H1,
        /// Level 2 heading (secure)
        H2,
        /// Level 3 heading (secure)
        H3,
        /// Level 4 heading (secure)
        H4,
        /// Level 5 heading (secure)
        H5,
        /// Level 6 heading (secure)
        H6,
        /// Highlight text
        Highlight,
        /// Horizontal rule (secure)
        Hr,
        /// Hyperlink (secure)
        Hyperlink,
        /// show image
        Image,
        /// italic
        Italic,
        /// play music
        Music,
        /// MXP command (eg. MXP OFF)
        Mxp,
        /// ignore next newline
        NoBr,
        /// Paragraph break (secure)
        P,
        /// send password
        Password,
        /// causes a new connect to open
        Relocate,
        /// close all open tags
        Reset,
        /// Soft line break
        SBr,
        /// eg. <send href="go west"> west
        Send,
        /// Small text
        Small,
        /// play sound
        Sound,
        /// status
        Stat,
        /// Strikethrough
        Strikeout,
        /// what commands we support
        Support,
        /// Non-proportional font
        Tt,
        /// underline
        Underline,
        /// send username
        User,
        /// Set variable
        Var,
        /// version request
        Version,
    }
}

impl ActionKind {
    /// Returns `true` if this is a command tag, i.e. a tag with no closing tag.
    pub const fn is_command(self) -> bool {
        matches!(
            self,
            Self::Br
                | Self::Expire
                | Self::Filter
                | Self::Gauge
                | Self::Hr
                | Self::Music
                | Self::Mxp
                | Self::NoBr
                | Self::Password
                | Self::Relocate
                | Self::Reset
                | Self::SBr
                | Self::Stat
                | Self::Support
                | Self::User
                | Self::Version
                | Self::Frame
                | Self::Image
                | Self::Sound
        )
    }

    /// Returns `true` if the action can be used if the MXP [`Mode`](crate::Mode) is "open".
    pub const fn is_open(self) -> bool {
        matches!(
            self,
            Self::Bold
                | Self::Color
                | Self::Italic
                | Self::Highlight
                | Self::Strikeout
                | Self::Small
                | Self::Tt
                | Self::Underline
                | Self::Font
        )
    }
}
