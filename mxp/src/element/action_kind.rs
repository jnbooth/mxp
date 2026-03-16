use flagset::flags;

flags! {
    /// Type of effect caused by an [`Action`](crate::Action).
    pub enum ActionKind: u64 {
        /// [`<BOLD>`](https://www.zuggsoft.com/zmud/mxp.htm#Text%20Formatting):
        /// Make text bold.
        Bold,
        /// [`<BR>`](https://www.zuggsoft.com/zmud/mxp.htm#Line%20Spacing):
        /// Insert a hard line break.
        Br,
        ///[`<COLOR>`](https://www.zuggsoft.com/zmud/mxp.htm#Text%20Formatting):
        /// Change text color.
        Color,
        /// [`<DEST>`](https://www.zuggsoft.com/zmud/mxp.htm#Cursor%20Control):
        /// Set destination frame.
        Dest,
        /// [`<EXPIRE>`](https://www.zuggsoft.com/zmud/mxp.htm#Links):
        /// Expire links.
        Expire,
        /// [`<FILTER>`](https://www.zuggsoft.com/zmud/mxp.htm#File%20Filters):
        /// Set file filter.
        Filter,
        /// [`<FONT>`](https://www.zuggsoft.com/zmud/mxp.htm#Text%20Formatting):
        /// Change text font.
        Font,
        /// [`<FRAME>`](https://www.zuggsoft.com/zmud/mxp.htm#Frames):
        /// Create a frame window.
        Frame,
        /// [`<GAUGE>`](https://www.zuggsoft.com/zmud/mxp.htm#Using%20Entities):
        /// Display an MXP entity value as a gauge.
        Gauge,
        /// Level 1 heading.
        H1,
        /// Level 2 heading.
        H2,
        /// Level 3 heading.
        H3,
        /// Level 4 heading.
        H4,
        /// Level 5 heading.
        H5,
        /// Level 6 heading.
        H6,
        /// [`<HIGH>`](https://www.zuggsoft.com/zmud/mxp.htm#Text%20Formatting):
        /// Highlight text.
        Highlight,
        /// [`<HR>`](https://www.zuggsoft.com/zmud/mxp.htm#HTML%20tags):
        /// Insert a horizontal rule.
        Hr,
        /// [`<A>`](https://www.zuggsoft.com/zmud/mxp.htm#Links):
        /// Hyperlink.
        Hyperlink,
        /// [`<IMAGE>`](https://www.zuggsoft.com/zmud/mxp.htm#Images):
        /// Display an image.
        Image,
        /// [`<ITALIC>`](https://www.zuggsoft.com/zmud/mxp.htm#Text%20Formatting):
        /// Make text italic.
        Italic,
        /// [`<MUSIC>`](https://www.zuggsoft.com/zmud/mxp.htm#MSP%20Compatibility):
        /// Play or stop music.
        Music,
        /// [`<MXP OFF>`](https://gpascal.com/forum/?id=232):
        /// MXP control command. This is an unofficial extension to the MXP protocol.
        Mxp,
        /// [`<NOBR>`](https://www.zuggsoft.com/zmud/mxp.htm#Line%20Spacing):
        /// Ignore next newline.
        NoBr,
        /// [`<P>`](https://www.zuggsoft.com/zmud/mxp.htm#Line%20Spacing):
        /// Insert a paragraph break.
        P,
        /// [`<PASSWORD>`](https://www.zuggsoft.com/zmud/mxp.htm#Crosslinking%20multiple%20MUD%20servers):
        /// Prompt client to send user password.
        Password,
        /// [`<RELOCATE>`](https://www.zuggsoft.com/zmud/mxp.htm#Crosslinking%20multiple%20MUD%20servers):
        /// Prompt client to switch to a new network connection.
        Relocate,
        /// [`<RESET>`](https://gpascal.com/forum/?id=232):
        /// Close all OPEN tags. This is an unofficial extension to the MXP protocol.
        Reset,
        /// [`<SBR>`](https://www.zuggsoft.com/zmud/mxp.htm#Line%20Spacing):
        /// Insert a soft linebreak.
        SBr,
        /// [`<Send>`](https://www.zuggsoft.com/zmud/mxp.htm#Links):
        /// Turn text into a link that sends a command to the world.
        Send,
        /// [`<SMALL>`](https://www.zuggsoft.com/zmud/mxp.htm#HTML%20tags):
        /// Display text in a smaller size.
        Small,
        /// [`<SOUND>`](https://www.zuggsoft.com/zmud/mxp.htm#MSP%20Compatibility):
        /// Play or stop a sound file.
        Sound,
        /// [`<STAT>`](https://www.zuggsoft.com/zmud/mxp.htm#Using%20Entities):
        /// Display an MXP entity value on the status bar.
        Stat,
        /// [`<STRIKEOUT>`](https://www.zuggsoft.com/zmud/mxp.htm#Text%20Formatting):
        /// Strike-out the text.
        Strikeout,
        /// [`<SUPPORT>`](https://www.zuggsoft.com/zmud/mxp.htm#Version%20Control):
        /// Prompt client to respond with the commands that it supports.
        Support,
        /// [`<TT>`](https://www.zuggsoft.com/zmud/mxp.htm#HTML%20tags):
        /// Display text in a non-proportional font.
        Tt,
        /// [`<UNDERLINE>`](https://www.zuggsoft.com/zmud/mxp.htm#Text%20Formatting):
        /// Underline text.
        Underline,
        /// [`<USER>`](https://www.zuggsoft.com/zmud/mxp.htm#Crosslinking%20multiple%20MUD%20servers):
        /// Prompt client to send username.
        User,
        /// [`<VAR>`](https://www.zuggsoft.com/zmud/mxp.htm#ENTITY):
        /// Set an MXP variable.
        Var,
        /// [`<VERSION>`](https://www.zuggsoft.com/zmud/mxp.htm#Version%20Control):
        /// Prompt client to respond with its client and version of MXP.
        Version,
    }
}

impl ActionKind {
    /// Returns `true` if this is a command tag. Command tags do not have content, so they have no
    /// closing tag.
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

    /// Returns `true` if the action can be used if the MXP line [`Mode`](crate::Mode) is OPEN.
    ///
    /// The following actions are OPEN:
    ///
    /// - [`Bold`](Self::Bold)
    /// - [`Color`](Self::Color)
    /// - [`Font`](Self::Font)
    /// - [`Highlight`](Self::Highlight)
    /// - [`Italic`](Self::Italic)
    /// - [`Small`](Self::Small)
    /// - [`Strikeout`](Self::Strikeout)
    /// - [`Tt`](Self::Tt)
    /// - [`Underline`](Self::Underline)
    pub const fn is_open(self) -> bool {
        matches!(
            self,
            Self::Bold
                | Self::Color
                | Self::Font
                | Self::Highlight
                | Self::Italic
                | Self::Small
                | Self::Strikeout
                | Self::Tt
                | Self::Underline
        )
    }
}
