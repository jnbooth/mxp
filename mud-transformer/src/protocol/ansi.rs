use std::time::Duration;
use std::{iter, slice};

use mxp::RgbColor;
use mxp::escape::ansi;

use crate::input::BufferedInput;
use crate::output::{BufferedOutput, ControlFragment, TextStyle};
use crate::responses::{
    OkReport, PrimaryAttributeReport, SecondaryAttributeReport, SecureResetConfirmation,
    TerminalParamsReport,
};
use crate::term::{
    self, AttributeRequest, CursorEffect, CursorStyle, DeviceStatus, EraseRange, EraseTarget,
    HighlightTracking, KeyboardLed, LocatorReporting, LocatorUnit, Mode, PrintFunction, Rect,
    RectEffect, RefreshRate, Reset, ReverseVisualCharacterAttribute, StatusDisplayType, TabEffect,
    TermColor, VisualCharacterAttribute, WindowOp,
};

enum Palette {
    Foreground,
    Background,
}

impl Palette {
    pub fn set(self, output: &mut BufferedOutput, color: RgbColor) {
        match self {
            Self::Background => output.set_ansi_background(color),
            Self::Foreground => output.set_ansi_foreground(color),
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
            Self::Background => output.set_ansi_background(TermColor::Unset),
            Self::Foreground => output.set_ansi_foreground(TermColor::Unset),
        }
    }

    fn interpret_mode<I>(self, output: &mut BufferedOutput, mut iter: I) -> Option<()>
    where
        I: Iterator<Item = Option<u8>>,
    {
        match iter.next()?? {
            ansi::BEGIN_XTERM_COLOR => {
                self.set(output, RgbColor::xterm(iter.next()??));
                Some(())
            }
            ansi::BEGIN_TRUECOLOR => {
                let color = RgbColor {
                    r: iter.next()??,
                    g: iter.next()??,
                    b: iter.next()??,
                };
                self.set(output, color);
                Some(())
            }
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Outcome {
    Fail,
    Continue,
    Done,
    Mxp(mxp::Mode),
}

#[derive(Clone, Debug)]
pub(crate) struct Interpreter {
    code: Option<u16>,
    sequence: Vec<Option<u16>>,
    prefix: u8,
    suffix: u8,

    pub margins: term::Rect,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

#[inline]
const fn unwrap_pos(value: Option<u16>) -> u16 {
    match value {
        None | Some(0) => 1,
        Some(value) => value,
    }
}

impl Interpreter {
    pub const fn new() -> Self {
        Self {
            code: None,
            sequence: Vec::new(),
            prefix: 0,
            suffix: 0,
            margins: Rect::new(),
        }
    }

    pub fn reset(&mut self) {
        self.code = None;
        self.sequence.clear();
        self.prefix = 0;
        self.suffix = 0;
    }

    pub fn interpret(
        &mut self,
        code: u8,
        output: &mut BufferedOutput,
        input: &mut BufferedInput,
    ) -> Outcome {
        self.try_interpret(code, output, input)
            .unwrap_or(Outcome::Fail)
    }

    fn iter(&mut self) -> iter::Copied<slice::Iter<'_, Option<u16>>> {
        if self.sequence.is_empty() {
            slice::from_ref(&self.code)
        } else {
            self.sequence.push(self.code);
            self.sequence.as_slice()
        }
        .iter()
        .copied()
    }

    #[inline]
    const fn code_or(&self, default: u16) -> u16 {
        match self.code {
            Some(n) => n,
            None => default,
        }
    }

    #[inline]
    const fn code_u8(&self) -> Option<u8> {
        match self.code {
            #[allow(clippy::cast_possible_truncation)]
            Some(n) if n <= u8::MAX as u16 => Some(n as u8),
            _ => None,
        }
    }

    #[inline]
    const fn code_u8_or(&self, default: u8) -> Option<u8> {
        match self.code {
            #[allow(clippy::cast_possible_truncation)]
            Some(n) if n <= u8::MAX as u16 => Some(n as u8),
            None => Some(default),
            _ => None,
        }
    }

    // https://tintin.mudhalla.net/info/vt100/
    // https://www.xfree86.org/current/ctlseqs.html
    fn try_interpret(
        &mut self,
        code: u8,
        output: &mut BufferedOutput,
        input: &mut BufferedInput,
    ) -> Option<Outcome> {
        if self.suffix != 0 {
            self.interpret_suffix(code, output, input)?;
            return Some(Outcome::Done);
        }
        if code < 0x20 {
            return None;
        }
        if code < 0x30 {
            self.suffix = code;
            return Some(Outcome::Continue);
        }
        if code < 0x3A {
            let digit = u16::from(code - b'0');
            self.code = Some(match self.code {
                Some(n) => n.checked_mul(10)?.checked_add(digit)?,
                None => digit,
            });
            return Some(Outcome::Continue);
        }
        if code < 0x3C {
            self.sequence.push(self.code.take());
            return Some(Outcome::Continue);
        }
        if self.prefix != 0 {
            output.append(match self.prefix {
                b'>' => self.interpret_secondary(code, input),
                b'?' => return self.interpret_private(code, output),
                _ => None,
            }?);
            return Some(Outcome::Done);
        }
        let single = self.sequence.is_empty();
        let empty = single && self.code.is_none();
        if code < 0x40 {
            if !empty {
                return None;
            }
            self.prefix = code;
            return Some(Outcome::Continue);
        }
        if !single {
            return self.interpret_sequence(code, output);
        }
        let n = unwrap_pos(self.code);
        output.append(match code {
            b'@' => ControlFragment::InsertSpaces(n.into()),
            b'A' => CursorEffect::Up(n).into(),
            b'B' => CursorEffect::Down(n).into(),
            b'C' => CursorEffect::Forward(n).into(),
            b'D' => CursorEffect::Back(n).into(),
            b'E' => CursorEffect::NextLine(n).into(),
            b'F' => CursorEffect::PreviousLine(n).into(),
            b'G' => CursorEffect::HorizontalAbsolute(n).into(),
            b'I' => CursorEffect::TabForward(n).into(),
            b'J' => self.interpret_erase(EraseTarget::Display, false)?,
            b'K' => self.interpret_erase(EraseTarget::Line, false)?,
            b'L' => ControlFragment::InsertLines(n.into()),
            b'M' => ControlFragment::DeleteLines(n.into()),
            b'P' => ControlFragment::DeleteCharacters(n.into()),
            b'S' => CursorEffect::ScrollUp(n).into(),
            b'T' => CursorEffect::ScrollDown(n).into(),
            b'U' => CursorEffect::NextPage(n.into()).into(),
            b'V' => CursorEffect::PrecedingPage(n.into()).into(),
            b'X' => ControlFragment::EraseCharacters(n.into()),
            b'Z' => CursorEffect::TabBack(n).into(),
            b'`' => CursorEffect::ColumnAbsolute(n).into(),
            b'a' => CursorEffect::ColumnRelative(n).into(),
            b'b' => ControlFragment::Repeat(n.into()),
            b'c' => {
                if !matches!(self.code, None | Some(0)) {
                    return None;
                }
                write!(input, "{PrimaryAttributeReport}");
                return Some(Outcome::Done);
            }
            b'd' => CursorEffect::RowAbsolute(n).into(),
            b'e' => CursorEffect::RowRelative(n).into(),
            b'g' => match self.code {
                None | Some(0) => TabEffect::ClearAtCursor.into(),
                Some(3) => TabEffect::ClearAll.into(),
                _ => return None,
            },
            b'h' => return self.set_modes(true, output),
            b'i' => ControlFragment::MediaCopy(PrintFunction::new(self.code_u8()?, false)),
            b'l' => return self.set_modes(false, output),
            b'm' => {
                self.interpret_mode(output)?;
                return Some(Outcome::Done);
            }
            b'n' => {
                if self.code == Some(5) {
                    write!(input, "{OkReport}");
                    return Some(Outcome::Done);
                }
                ControlFragment::DeviceStatusReport(DeviceStatus::new(self.code_u8()?, false))
            }
            b'q' => match self.code {
                Some(0) => ControlFragment::SetLed(KeyboardLed::All, false),
                Some(1) => ControlFragment::SetLed(KeyboardLed::NumLock, true),
                Some(2) => ControlFragment::SetLed(KeyboardLed::CapsLock, true),
                Some(3) => ControlFragment::SetLed(KeyboardLed::ScrollLock, true),
                Some(4) => ControlFragment::SetLed(KeyboardLed::NumLock, false),
                Some(5) => ControlFragment::SetLed(KeyboardLed::CapsLock, false),
                Some(6) => ControlFragment::SetLed(KeyboardLed::ScrollLock, false),
                _ => return None,
            },
            b'r' => ControlFragment::ModeRestore(Mode::new(self.code?, false)),
            b's' => {
                if empty {
                    CursorEffect::Save { dec: false }.into()
                } else {
                    ControlFragment::ModeSave(Mode::new(self.code?, false))
                }
            }
            b't' => {
                if empty {
                    WindowOp::SetLines(24).into()
                } else {
                    WindowOp::parse(self.iter()).next()?.into()
                }
            }
            b'u' => {
                if !empty {
                    return None;
                }
                CursorEffect::Restore { dec: false }.into()
            }
            b'x' => {
                if !matches!(self.code, None | Some(0 | 1)) {
                    return None;
                }
                write!(input, "{TerminalParamsReport}");
                return Some(Outcome::Done);
            }
            b'z' => return Some(Outcome::Mxp(self.code?.try_into().ok()?)),
            _ => return None,
        });
        Some(Outcome::Done)
    }

    fn interpret_sequence(&mut self, code: u8, output: &mut BufferedOutput) -> Option<Outcome> {
        output.append(match code {
            b'H' | b'f' => {
                let &[row] = self.exact_sequence()?;
                CursorEffect::Position {
                    row: unwrap_pos(row),
                    column: unwrap_pos(self.code),
                }
                .into()
            }
            b'T' => {
                let &[func, start_x, start_y, first_row] = self.exact_sequence()?;
                HighlightTracking {
                    func: func.unwrap_or_default() != 0,
                    start_x: unwrap_pos(start_x),
                    start_y: unwrap_pos(start_y),
                    first_row: unwrap_pos(first_row),
                    last_row: unwrap_pos(self.code),
                }
                .into()
            }
            b'h' => return self.set_modes(true, output),
            b'l' => return self.set_modes(false, output),
            b'm' => {
                self.interpret_mode(output)?;
                return Some(Outcome::Done);
            }
            b'r' => {
                let &[top] = self.exact_sequence()?;
                self.margins.top = top;
                self.margins.bottom = self.code;
                ControlFragment::VMargins {
                    top,
                    bottom: self.code,
                }
            }
            b's' => {
                let &[left] = self.exact_sequence()?;
                self.margins.left = left;
                self.margins.right = self.code;
                ControlFragment::HMargins {
                    left,
                    right: self.code,
                }
            }
            b't' => {
                for op in WindowOp::parse(self.iter()) {
                    output.append(op);
                }
                return Some(Outcome::Done);
            }
            b'~' => {
                let &[keystroke] = self.exact_sequence()?;
                output.append(ControlFragment::FunctionKey {
                    keystroke: keystroke?.try_into().ok()?,
                    modifiers: self.code_u8_or(0)?,
                });
                return Some(Outcome::Done);
            }
            _ => return None,
        });
        Some(Outcome::Done)
    }

    fn set_modes(&mut self, enable: bool, output: &mut BufferedOutput) -> Option<Outcome> {
        let private = self.prefix == b'?';
        for mode in self.iter() {
            output.append(ControlFragment::ModeSet(Mode::new(mode?, private), enable));
        }
        Some(Outcome::Done)
    }

    fn interpret_private(&mut self, code: u8, output: &mut BufferedOutput) -> Option<Outcome> {
        output.append(match code {
            b'h' => return self.set_modes(true, output),
            b'l' => return self.set_modes(false, output),
            _ if !self.sequence.is_empty() => return None,
            b'J' => self.interpret_erase(EraseTarget::Display, true)?,
            b'K' => self.interpret_erase(EraseTarget::Line, true)?,
            b'W' if self.code == Some(5) => TabEffect::SetEvery8Columns.into(),
            b'g' => ControlFragment::QueryKeyFormat(self.code_u8()?),
            b'i' => ControlFragment::MediaCopy(PrintFunction::new(self.code_u8()?, true)),
            b'n' => ControlFragment::DeviceStatusReport(DeviceStatus::new(self.code_u8()?, true)),
            b'r' => ControlFragment::ModeRestore(Mode::new(self.code?, true)),
            b's' => ControlFragment::ModeSave(Mode::new(self.code?, true)),
            _ => return None,
        });
        Some(Outcome::Done)
    }

    fn interpret_suffix(
        &mut self,
        code: u8,
        output: &mut BufferedOutput,
        input: &mut BufferedInput,
    ) -> Option<()> {
        output.append(match &[self.suffix, code] {
            b"$p" => ControlFragment::ModeRequest(Mode::new(self.code?, self.prefix == b'?')),
            b"$r" => {
                self.sequence.push(self.code);
                let (rect, attributes) = self.sequence_rect()?;
                for &attribute in attributes {
                    let attribute = VisualCharacterAttribute::from_code(attribute?)?;
                    output.append(RectEffect::SetAttributes(attribute).with(rect));
                }
                return Some(());
            }
            b"$t" => {
                self.sequence.push(self.code);
                let (rect, attributes) = self.sequence_rect()?;
                for &attribute in attributes {
                    let attribute = ReverseVisualCharacterAttribute::from_code(attribute?)?;
                    output.append(RectEffect::ReverseAttributes(attribute).with(rect));
                }
                return Some(());
            }
            b"$v" => {
                let (rect, &[sp, td, tl]) = self.exact_sequence_rect()?;
                RectEffect::Copy {
                    source: unwrap_pos(sp).into(),
                    row: unwrap_pos(td),
                    column: unwrap_pos(tl),
                    target: unwrap_pos(self.code).into(),
                }
                .with(rect)
            }
            b"$x" => {
                let &[c, t, l, b] = self.exact_sequence()?;
                let fill_char = c?.try_into().ok()?;
                RectEffect::Fill { fill_char }.with(Rect {
                    top: t,
                    left: l,
                    bottom: b,
                    right: self.code,
                })
            }
            b"$z" | b"${" => {
                self.sequence.push(self.code);
                let (rect, &[]) = self.exact_sequence_rect()?;
                RectEffect::Erase {
                    selective: code == b'{',
                }
                .with(rect)
            }
            b"'w" => {
                self.sequence.push(self.code);
                let (rect, &[]) = self.exact_sequence_rect()?;
                RectEffect::Filter.with(rect)
            }
            b"'z" => {
                let &[param] = self.exact_sequence()?;
                let reporting = LocatorReporting::from_code(param)?;
                let unit = LocatorUnit::from_code(param)?;
                ControlFragment::SetLocator(reporting, unit)
            }
            b"'{" => {
                let mut on_press = false;
                let mut on_release = false;
                for event in self.iter() {
                    match event {
                        None => (),
                        Some(0) => {
                            on_press = false;
                            on_release = false;
                        }
                        Some(1) => on_press = true,
                        Some(2) => on_press = false,
                        Some(3) => on_release = true,
                        Some(4) => on_release = false,
                        _ => return None,
                    }
                }
                ControlFragment::SelectLocatorEvents {
                    on_press,
                    on_release,
                }
            }
            b",p" => {
                let &[hour] = self.exact_sequence()?;
                let hour = hour.unwrap_or(8);
                let minute = self.code_or(0);
                if hour > 23 || minute > 59 {
                    return None;
                }
                #[allow(clippy::cast_possible_truncation)]
                ControlFragment::TimeOfDay {
                    hour: hour as u8,
                    minute: minute as u8,
                }
            }
            _ if !self.sequence.is_empty() => return None,
            b" P" => CursorEffect::PageAbsolute(unwrap_pos(self.code).into()).into(),
            b" Q" => CursorEffect::PageForward(unwrap_pos(self.code).into()).into(),
            b" R" => CursorEffect::PageBackward(unwrap_pos(self.code).into()).into(),
            b" q" => ControlFragment::StyleCursor(CursorStyle::from_code(self.code)?),
            b" r" => ControlFragment::SetKeyClickVolume(self.code_u8_or(0)?),
            b" t" => ControlFragment::SetWarningVolume(self.code_u8_or(0)?),
            b" u" => ControlFragment::SetMarginVolume(self.code_u8_or(0)?),
            b"!p" if self.code.is_none() => ControlFragment::ResetTerminal(Reset::Soft),
            b"\"q" => ControlFragment::SetCharacterProtection(match self.code {
                None | Some(0 | 2) => false,
                Some(1) => true,
                _ => return None,
            }),
            b"\"t" => ControlFragment::SetRefreshRate(RefreshRate::from_code(self.code)?),
            b"\"v" => AttributeRequest::DisplayedExtent.into(),
            b"$|" => ControlFragment::SetColumns(self.code_or(80)),
            b"$q" => ControlFragment::SetDisconnectDelay(match self.code {
                None | Some(0 | 3) => Duration::from_secs(2),
                Some(1) => Duration::ZERO,
                Some(2) => Duration::from_millis(60),
                _ => return None,
            }),
            b"$u" => match self.code {
                Some(1) => AttributeRequest::TerminalState.into(),
                _ => return None,
            },
            b"$w" => match self.code {
                Some(1) => AttributeRequest::CursorInformation.into(),
                Some(2) => AttributeRequest::TabStop.into(),
                _ => return None,
            },
            b"$}" => ControlFragment::SelectStatusLine(match self.code {
                None | Some(0) => false,
                Some(1) => true,
                _ => return None,
            }),
            b"$~" => ControlFragment::SetStatusDisplay(StatusDisplayType::from_code(self.code)?),
            b"&u" => AttributeRequest::PreferredSupplementalSet.into(),
            b"'|" if self.code_or(0) <= 1 => AttributeRequest::LocatorPosition.into(),
            b"'}" => ControlFragment::InsertColumns(unwrap_pos(self.code).into()),
            b"'~" => ControlFragment::DeleteColumns(unwrap_pos(self.code).into()),
            b"*x" => ControlFragment::SetAttributeChangeExtent(match self.code {
                None | Some(0 | 1) => false,
                Some(2) => true,
                _ => return None,
            }),
            b"*|" => ControlFragment::SetRows(self.code?),
            b"+p" => {
                if let Some(sequence) = self.code {
                    write!(input, "{}", SecureResetConfirmation { sequence });
                }
                ControlFragment::ResetTerminal(Reset::Secure)
            }
            _ => return None,
        });
        Some(())
    }

    fn interpret_erase(&self, target: EraseTarget, selective: bool) -> Option<ControlFragment> {
        if self.code == Some(3) && target == EraseTarget::Display {
            return Some(ControlFragment::Clear);
        }
        Some(ControlFragment::Erase {
            target,
            range: EraseRange::from_code(self.code)?,
            selective,
        })
    }

    fn interpret_mode(&mut self, output: &mut BufferedOutput) -> Option<()> {
        let mut iter = self.iter().map(|n| n.and_then(|n| u8::try_from(n).ok()));

        while let Some(Some(code)) = iter.next() {
            match code {
                ansi::RESET => output.reset_ansi(),

                ansi::BOLD => output.set_ansi_flag(TextStyle::Bold),
                ansi::FAINT => output.set_ansi_flag(TextStyle::Faint),
                ansi::ITALIC => output.set_ansi_flag(TextStyle::Italic),
                ansi::UNDERLINE => output.set_ansi_flag(TextStyle::Underline),
                ansi::SLOW_BLINK | ansi::RAPID_BLINK => output.set_ansi_flag(TextStyle::Blink),
                ansi::INVERSE => output.set_ansi_flag(TextStyle::Inverse),
                ansi::CONCEAL => output.set_ansi_flag(TextStyle::Conceal),
                ansi::STRIKEOUT => output.set_ansi_flag(TextStyle::Strikeout),

                ansi::CANCEL_BOLD => {
                    output.unset_ansi_flag(TextStyle::Bold);
                    output.unset_ansi_flag(TextStyle::Faint);
                }
                ansi::CANCEL_ITALIC => output.unset_ansi_flag(TextStyle::Italic),
                ansi::CANCEL_UNDERLINE => output.unset_ansi_flag(TextStyle::Underline),
                ansi::CANCEL_BLINK => output.unset_ansi_flag(TextStyle::Blink),
                ansi::CANCEL_INVERSE => output.unset_ansi_flag(TextStyle::Inverse),
                ansi::CANCEL_CONCEAL => output.unset_ansi_flag(TextStyle::Conceal),
                ansi::CANCEL_STRIKEOUT => output.unset_ansi_flag(TextStyle::Strikeout),

                ansi::FG_256_COLOR => Palette::Foreground.interpret_mode(output, &mut iter)?,
                ansi::BG_256_COLOR => Palette::Background.interpret_mode(output, &mut iter)?,
                ansi::FG_BLACK..=ansi::FG_WHITE => Palette::Foreground.set_code(output, code),
                ansi::FG_DEFAULT => Palette::Foreground.set_default(output),
                ansi::BG_BLACK..=ansi::BG_WHITE => Palette::Background.set_code(output, code),
                ansi::BG_DEFAULT => Palette::Background.set_default(output),
                _ => (),
            }
        }
        Some(())
    }

    fn interpret_secondary(
        &mut self,
        code: u8,
        input: &mut BufferedInput,
    ) -> Option<ControlFragment> {
        match code {
            b'f' => {
                let (param, value) = match (&*self.sequence, self.code) {
                    (&[], None) => {
                        return Some(ControlFragment::ResetKeyFormat);
                    }
                    (&[], Some(param)) => (param, None),
                    (&[Some(param)], value) => (param, value),
                    _ => return None,
                };
                Some(ControlFragment::SetKeyFormat {
                    param: param.try_into().ok()?,
                    value,
                })
            }
            _ if !self.sequence.is_empty() => None,
            b'c' if self.code_or(0) == 0 => {
                write!(input, "{SecondaryAttributeReport}");
                None
            }
            b's' => {
                let enable = match self.code {
                    None | Some(0) => false,
                    Some(1) => true,
                    _ => return None,
                };
                Some(ControlFragment::SetShiftEscape(enable))
            }
            _ => None,
        }
    }

    fn exact_sequence<const N: usize>(&self) -> Option<&[Option<u16>; N]> {
        self.sequence.as_slice().try_into().ok()
    }

    const fn sequence_rect(&self) -> Option<(Rect, &[Option<u16>])> {
        let [t, l, b, r, remaining @ ..] = self.sequence.as_slice() else {
            return None;
        };
        let rect = Rect {
            top: *t,
            left: *l,
            bottom: *b,
            right: *r,
        };
        Some((rect, remaining))
    }

    fn exact_sequence_rect<const N: usize>(&self) -> Option<(Rect, &[Option<u16>; N])> {
        let [t, l, b, r, remaining @ ..] = self.sequence.as_slice() else {
            return None;
        };
        let remaining = remaining.try_into().ok()?;
        let rect = Rect {
            top: *t,
            left: *l,
            bottom: *b,
            right: *r,
        };
        Some((rect, remaining))
    }
}
