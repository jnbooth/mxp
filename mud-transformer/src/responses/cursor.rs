use std::fmt;

use flagset::{FlagSet, flags};

use crate::ControlFragment;
use crate::escape::ansi::{DCS, ST};
use crate::output::{BufferedOutput, TextStyle};
use crate::term::{CursorEffect, Mode};

const BIT_7: u8 = 0b1000000;

flags! {
    #[derive(PartialOrd, Ord, Hash)]
    pub enum CursorInformation: u16 {
        /// Bold is on.
        Bold,
        /// Underline is on.
        Underline,
        /// Blinking is on.
        Blink,
        /// Reverse video is on.
        Inverse,

        /// Selective erase (DECSCA) is on.
        SelectiveErase,

        /// Origin mode.
        Origin,
        /// Single shift 2: G2 is mapped into GL for the next typed character only.
        SS2,
        /// Single shift 3: G3 is mapped into GL for the next typed character only.
        SS3,
        /// Autowrap pending.
        Autowrap,

        /// G0 character set has 96 characters (default 94).
        G0_96,
        /// G1 character set has 96 characters (default 94).
        G1_96,
        /// G2 character set has 96 characters (default 94).
        G2_96,
        /// G3 character set has 96 characters (default 94).
        G3_96,
    }
}

impl CursorInformation {
    const REND: &[Self] = &[Self::Bold, Self::Underline, Self::Blink, Self::Inverse];
    const ATT: &[Self] = &[Self::SelectiveErase];
    const FLAG: &[Self] = &[Self::Origin, Self::SS2, Self::SS3, Self::Autowrap];
    const CSS: &[Self] = &[Self::G0_96, Self::G1_96, Self::G2_96, Self::G3_96];

    const fn bit(self) -> u8 {
        match self {
            Self::Bold | Self::SelectiveErase | Self::Origin | Self::G0_96 => 0b1,
            Self::Underline | Self::SS2 | Self::G1_96 => 0b10,
            Self::Blink | Self::SS3 | Self::G2_96 => 0b100,
            Self::Inverse | Self::Autowrap | Self::G3_96 => 0b1000,
        }
    }

    const fn mode(self) -> Option<Mode> {
        match self {
            Self::Origin => Some(Mode::ORIGIN),
            Self::Autowrap => Some(Mode::AUTOWRAP),
            _ => None,
        }
    }

    const fn style(self) -> Option<TextStyle> {
        match self {
            Self::Bold => Some(TextStyle::Bold),
            Self::Underline => Some(TextStyle::Underline),
            Self::Blink => Some(TextStyle::Blink),
            Self::Inverse => Some(TextStyle::Inverse),
            _ => None,
        }
    }

    fn flags(bytes: &str, targets: &[Self]) -> FlagSet<Self> {
        let mut flags = FlagSet::empty();
        for &c in bytes.as_bytes() {
            for &info in targets {
                if (c & info.bit()) != 0 {
                    flags |= info;
                }
            }
        }
        flags
    }
}

/// Formats a DECCIR response.
pub struct CursorInformationReport<'a> {
    /// Number of the line the cursor is on.
    pub row: u16,
    /// Number of the column the cursor is at.
    pub column: u16,
    /// Number of the current page.
    pub page: usize,
    pub flags: FlagSet<CursorInformation>,
    /// Number of the logical character set (G0 through G3) mapped into GL.
    pub gl: u8,
    /// Number of the logical character set (G0 through G3) mapped into GR.
    pub gr: u8,
    /// String of intermediate and final characters indicating the character sets designated as G0
    /// through G3. These final characters are the same as those used in select character set (SCS)
    /// sequences.
    pub desig: &'a str,
}

impl fmt::Display for CursorInformationReport<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let rend = self.rend();
        let att = self.att();
        let flag = self.flag();
        let css = self.css();
        let Self {
            row,
            column,
            page,
            flags: _,
            gl,
            gr,
            desig,
        } = self;
        write!(
            f,
            "{DCS}1$u{row};{column};{page};{rend};{att};{flag};{gl};{gr};{css};{desig}{ST}"
        )
    }
}

impl<'a> CursorInformationReport<'a> {
    pub(crate) fn decode(s: &'a str) -> Option<Self> {
        let mut iter = s.split(';');
        let row = iter.next()?.parse().ok()?;
        let column = iter.next()?.parse().ok()?;
        let page = iter.next()?.parse().ok()?;
        let rend = iter.next()?;
        let att = iter.next()?;
        let flag = iter.next()?;
        let gl = iter.next()?.parse().ok()?;
        let gr = iter.next()?.parse().ok()?;
        let css = iter.next()?;
        let desig = iter.next()?;

        let flags = CursorInformation::flags(rend, CursorInformation::REND)
            | CursorInformation::flags(att, CursorInformation::ATT)
            | CursorInformation::flags(flag, CursorInformation::FLAG)
            | CursorInformation::flags(css, CursorInformation::CSS);

        Some(Self {
            row,
            column,
            page,
            flags,
            gl,
            gr,
            desig,
        })
    }

    pub(crate) fn restore(&self, output: &mut BufferedOutput) {
        output.append(CursorEffect::Position {
            row: self.row,
            column: self.column,
        });
        for info in FlagSet::<CursorInformation>::full() {
            let set = self.flags.contains(info);
            if let Some(mode) = info.mode() {
                output.append(ControlFragment::ModeSet(mode, set));
            }
            let Some(style) = info.style() else {
                continue;
            };
            if self.flags.contains(info) {
                output.set_ansi_flag(style);
            } else {
                output.unset_ansi_flag(style);
            }
        }
    }

    fn bit(&self, info: CursorInformation) -> u8 {
        if self.flags.contains(info) {
            info.bit()
        } else {
            0
        }
    }

    fn rend(&self) -> u8 {
        use CursorInformation::{Blink, Bold, Inverse, Underline};
        BIT_7 | self.bit(Bold) | self.bit(Underline) | self.bit(Blink) | self.bit(Inverse)
    }

    fn att(&self) -> u8 {
        use CursorInformation::SelectiveErase;
        BIT_7 | self.bit(SelectiveErase)
    }

    fn flag(&self) -> u8 {
        use CursorInformation::{Autowrap, Origin, SS2, SS3};
        BIT_7 | self.bit(Origin) | self.bit(SS2) | self.bit(SS3) | self.bit(Autowrap)
    }

    fn css(&self) -> u8 {
        use CursorInformation::{G0_96, G1_96, G2_96, G3_96};
        BIT_7 | self.bit(G0_96) | self.bit(G1_96) | self.bit(G2_96) | self.bit(G3_96)
    }
}
