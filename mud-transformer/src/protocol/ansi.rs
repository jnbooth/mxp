use mxp::RgbColor;
use mxp::escape::ansi;

use crate::output::{BufferedOutput, TermColor, TextStyle};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum Palette {
    Foreground,
    Background,
}

impl Palette {
    pub fn set(self, output: &mut BufferedOutput, color: RgbColor) {
        match self {
            Self::Background => output.set_ansi_foreground(color),
            Self::Foreground => output.set_ansi_background(color),
        }
    }

    pub fn set_code(self, output: &mut BufferedOutput, color: u8) {
        match self {
            Self::Background => output.set_ansi_background(TermColor::Ansi(color - ansi::BG_BLACK)),
            Self::Foreground => output.set_ansi_foreground(TermColor::Ansi(color - ansi::FG_BLACK)),
        }
    }

    pub fn set_default(self, output: &mut BufferedOutput) {
        match self {
            Self::Background => output.set_ansi_background(TermColor::BLACK),
            Self::Foreground => output.set_ansi_foreground(TermColor::WHITE),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Phase {
    Code,
    Start,
    FinishAnsi,
    Red,
    Green,
    Blue,
    Finish256,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum Outcome {
    Continue,
    Done,
    Mxp(mxp::Mode),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Interpreter {
    palette: Palette,
    phase: Phase,
    color: RgbColor,
    ansi_code: u8,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    pub const fn new() -> Self {
        Self {
            palette: Palette::Foreground,
            phase: Phase::Start,
            color: RgbColor::rgb(0, 0, 0),
            ansi_code: 0,
        }
    }

    pub fn reset(&mut self) {
        self.phase = Phase::Code;
        self.ansi_code = 0;
    }

    fn start(&mut self, palette: Palette) {
        self.palette = palette;
        self.phase = Phase::Start;
        self.color = RgbColor::rgb(0, 0, 0);
        self.ansi_code = 0;
    }

    pub fn interpret(&mut self, code: u8, output: &mut BufferedOutput) -> Outcome {
        match code {
            b'm' => self.interpret_code(output),
            b';' | b':' => {
                self.interpret_code(output);
                self.ansi_code = 0;
                Outcome::Continue
            }
            b'z' => Outcome::Mxp(mxp::Mode(self.ansi_code)),
            b'0'..=b'9' => {
                self.ansi_code = ansi::append_digit_to_code(self.ansi_code, code);
                Outcome::Continue
            }
            _ => Outcome::Done,
        }
    }

    pub fn interpret_code(&mut self, output: &mut BufferedOutput) -> Outcome {
        match self.phase {
            Phase::Code => {
                self.interpret_ansi(output);
                Outcome::Done
            }
            Phase::Start => match self.ansi_code {
                5 => {
                    self.ansi_code = 0;
                    self.phase = Phase::FinishAnsi;
                    Outcome::Continue
                }
                2 => {
                    self.ansi_code = 0;
                    self.phase = Phase::Red;
                    Outcome::Continue
                }
                _ => Outcome::Done,
            },
            Phase::FinishAnsi => {
                self.palette.set(output, RgbColor::xterm(self.ansi_code));
                Outcome::Done
            }
            Phase::Red => {
                self.color.r = self.ansi_code;
                self.phase = Phase::Green;
                Outcome::Continue
            }
            Phase::Green => {
                self.color.g = self.ansi_code;
                self.phase = Phase::Blue;
                Outcome::Continue
            }
            Phase::Blue => {
                self.color.b = self.ansi_code;
                self.phase = Phase::Finish256;
                Outcome::Continue
            }
            Phase::Finish256 => {
                self.palette.set(output, self.color);
                Outcome::Done
            }
        }
    }

    fn interpret_ansi(&mut self, output: &mut BufferedOutput) {
        match self.ansi_code {
            ansi::RESET => output.reset_ansi(),

            ansi::BOLD => output.set_ansi_flag(TextStyle::Bold),
            ansi::BLINK | ansi::SLOW_BLINK | ansi::FAST_BLINK => {
                output.set_ansi_flag(TextStyle::Italic);
            }
            ansi::UNDERLINE => output.set_ansi_flag(TextStyle::Underline),
            ansi::INVERSE => output.set_ansi_flag(TextStyle::Inverse),
            ansi::STRIKEOUT => output.set_ansi_flag(TextStyle::Strikeout),

            ansi::CANCEL_BOLD => output.unset_ansi_flag(TextStyle::Bold),
            ansi::CANCEL_BLINK | ansi::CANCEL_SLOW_BLINK | ansi::CANCEL_FAST_BLINK => {
                output.unset_ansi_flag(TextStyle::Italic);
            }
            ansi::CANCEL_UNDERLINE => output.unset_ansi_flag(TextStyle::Underline),
            ansi::CANCEL_INVERSE => output.unset_ansi_flag(TextStyle::Inverse),
            ansi::CANCEL_STRIKEOUT => output.unset_ansi_flag(TextStyle::Strikeout),

            ansi::FG_256_COLOR => self.start(Palette::Foreground),
            ansi::BG_256_COLOR => self.start(Palette::Background),
            ansi::FG_DEFAULT => Palette::Foreground.set_default(output),
            ansi::BG_DEFAULT => Palette::Background.set_default(output),
            ansi::FG_BLACK..=ansi::FG_WHITE => Palette::Foreground.set_code(output, self.ansi_code),
            ansi::BG_BLACK..=ansi::BG_WHITE => Palette::Background.set_code(output, self.ansi_code),
            _ => (),
        }
    }
}
