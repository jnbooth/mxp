use bytes::{Bytes, BytesMut};
use bytestring::ByteString;
use mxp::RgbColor;
use mxp::escape::ansi;

pub(crate) use super::ansi::Outcome;
use crate::input::BufferedInput;
use crate::output::{BufferedOutput, ControlFragment};
use crate::responses::{
    CursorInformationReport, HMarginsReport, TabStopReport, UnknownSettingReport, VMarginsReport,
};
use crate::term::{
    ControlStringType, CursorEffect, DynamicColor, Line, Mode, Rect, Reset, SelectionData,
    TabEffect,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum Start {
    Done,
    Continue,
    BeginString,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
enum Phase {
    #[default]
    Normal,
    /// CSI (Control Sequence Introducer)
    Csi,
    /// SCS (Select Character Set)
    Charset,
    /// DCS (Device Control String),
    /// SOS (Start of String),
    /// OSC (Operating System Command),
    /// PM (Privacy Message),
    /// APC (Application Program Command)
    ControlString,
    /// SCODFK (Define Function Key)
    FunctionKey,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Interpreter {
    ansi: super::ansi::Interpreter,
    phase: Phase,
    answerback: Vec<u8>,
    code: u8,
    extension: Option<u8>,
    control_string: BytesMut,
}

impl Interpreter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn answerback(&self) -> &[u8] {
        &self.answerback
    }

    pub fn start(&mut self, code: u8, output: &mut BufferedOutput) -> Start {
        output.append(match code {
            b'[' => {
                self.phase = Phase::Csi;
                self.ansi.reset();
                return Start::Continue;
            }
            b'6' => CursorEffect::BackIndex.into(),
            b'7' => CursorEffect::Save { dec: true }.into(),
            b'8' => CursorEffect::Restore { dec: false }.into(),
            b'9' => CursorEffect::ForwardIndex.into(),
            b'=' => ControlFragment::ModeSet(Mode::NUMERIC_KEYPAD, false),
            b'>' => ControlFragment::ModeSet(Mode::NUMERIC_KEYPAD, true),
            b'D' => CursorEffect::Index.into(),
            b'E' => ControlFragment::NextLine,
            b'H' => TabEffect::SetStop.into(),
            b'M' => CursorEffect::ReverseIndex.into(),
            b'Q' => {
                self.phase = Phase::FunctionKey;
                return Start::Continue;
            }
            b'V' => ControlFragment::GuardedAreaStart,
            b'W' => ControlFragment::GuardedAreaEnd,
            b'c' => ControlFragment::ResetTerminal(Reset::Hard),
            b'l' => ControlFragment::SetMemoryLock(true),
            b'm' => ControlFragment::SetMemoryLock(false),
            b' ' | b'%' | b'#' => {
                self.phase = Phase::Normal;
                self.code = code;
                return Start::Continue;
            }
            b'P' | b']' | b'X' | b'^' | b'_' => {
                self.code = code;
                self.phase = Phase::ControlString;
                return Start::BeginString;
            }
            b'(' | b')' | b'*' | b'+' | b'-' | b'.' | b'/' => {
                self.code = code;
                self.phase = Phase::Charset;
                return Start::Continue;
            }
            _ => return Start::Done,
        });
        Start::Done
    }

    pub fn interpret(
        &mut self,
        code: u8,
        output: &mut BufferedOutput,
        input: &mut BufferedInput,
    ) -> Outcome {
        match self.phase {
            Phase::Csi => self.ansi.interpret(code, output, input),
            Phase::Normal => {
                if let Some(fragment) = self.interpret_code(code) {
                    output.append(fragment);
                }
                Outcome::Done
            }
            Phase::ControlString => self.interpret_control_string(code, output, input),
            Phase::Charset => self.interpret_charset(code),
            Phase::FunctionKey => self.interpret_function_key(code),
        }
    }

    fn interpret_code(&self, code: u8) -> Option<ControlFragment> {
        match &[self.code, code] {
            b"#3" => Some(Line::DoubleHeightTop.into()),
            b"#4" => Some(Line::DoubleHeightBottom.into()),
            b"#5" => Some(Line::SingleWidth.into()),
            b"#6" => Some(Line::DoubleWidth.into()),
            b"#8" => Some(ControlFragment::ScreenAlignmentTest),
            _ => None,
        }
    }

    fn interpret_function_key(&mut self, code: u8) -> Outcome {
        if self.code == 0 {
            self.code = code;
            return Outcome::Continue;
        }
        let Some(delim) = self.extension else {
            self.extension = Some(code);
            return Outcome::Continue;
        };
        if code != delim {
            return Outcome::Continue;
        }
        self.code = 0;
        self.extension = None;
        Outcome::Done
    }

    fn interpret_charset(&mut self, code: u8) -> Outcome {
        if self.extension.is_none() && matches!(code, b'%' | b'"' | b'&') {
            self.extension = Some(code);
            return Outcome::Continue;
        }
        self.code = 0;
        self.extension = None;
        Outcome::Done
    }

    fn interpret_control_string(
        &mut self,
        code: u8,
        output: &mut BufferedOutput,
        input: &mut BufferedInput,
    ) -> Outcome {
        if code == ansi::BEL {
            self.finish_control_string(output, input);
            return Outcome::Done;
        }
        if code == ansi::ESC_ST && self.control_string.last() == Some(&ansi::ESC) {
            self.control_string.truncate(self.control_string.len() - 1);
            self.finish_control_string(output, input);
            return Outcome::Done;
        }
        self.control_string.extend_from_slice(&[code]);
        Outcome::Continue
    }

    fn finish_control_string(
        &mut self,
        output: &mut BufferedOutput,
        input: &mut BufferedInput,
    ) -> Option<()> {
        let control_string = self.control_string.split().freeze();
        let code = self.code;
        self.code = 0;
        let string_type = match code {
            ansi::ESC_DCS => return self.process_dcs(&control_string, output, input),
            ansi::ESC_OSC => return process_osc(&control_string, output, input),
            ansi::ESC_SOS => ControlStringType::Sos,
            ansi::ESC_PM => ControlStringType::Pm,
            ansi::ESC_APC => ControlStringType::Apc,
            _ => return None,
        };
        output.append(ControlFragment::ControlString(string_type, control_string));
        Some(())
    }

    fn process_dcs(
        &mut self,
        control_string: &Bytes,
        output: &mut BufferedOutput,
        input: &mut BufferedInput,
    ) -> Option<()> {
        let (code, command, rest) = parse_dcs(control_string)?;
        match command {
            b"v" if code == Some(1) => {
                self.answerback.clear();
                decode_hex(rest, &mut self.answerback)
            }
            b"$t" => process_restore_presentation_state(code.unwrap_or(0), rest, output),
            _ if code.is_some() => None,
            b"$q" => self.process_request_status(rest, output, input),
            b"+q" => process_request_term(rest, input),
            _ => None,
        }?;
        Some(())
    }

    fn process_request_status(
        &self,
        control_string: &[u8],
        output: &mut BufferedOutput,
        input: &mut BufferedInput,
    ) -> Option<()> {
        let control_string = str::from_utf8(control_string).ok()?;
        match control_string {
            "m" => write!(input, "{}", output.ansi_mode()),
            "r" => {
                let Rect { top, bottom, .. } = self.ansi.margins;
                write!(input, "{}", VMarginsReport { top, bottom });
            }
            "s" => {
                let Rect { left, right, .. } = self.ansi.margins;
                write!(input, "{}", HMarginsReport { left, right });
            }
            _ => write!(input, "{UnknownSettingReport}"),
        }
        Some(())
    }
}

/// Request Termcap/Terminfo String
#[allow(clippy::unnecessary_wraps)]
fn process_request_term(control_string: &[u8], input: &mut BufferedInput) -> Option<()> {
    input.append(ansi::DCS);
    input.append("0+r");
    input.append(control_string);
    input.append(ansi::ST);
    Some(())
}

fn process_restore_presentation_state(
    code: u8,
    control_string: &[u8],
    output: &mut BufferedOutput,
) -> Option<()> {
    let control_string = str::from_utf8(control_string).ok()?;
    match code {
        1 => CursorInformationReport::decode(control_string)?.restore(output),
        2 => TabStopReport::decode(control_string).restore(output),
        _ => return None,
    }
    Some(())
}

fn parse_dcs(control_string: &[u8]) -> Option<(Option<u8>, &[u8], &[u8])> {
    let (code, rest) = if control_string.first()?.is_ascii_digit() {
        let mut code = 0u8;
        let mut iter = control_string.iter();
        for &c in &mut iter {
            if !c.is_ascii_digit() {
                break;
            }
            code = code.checked_mul(10)?.checked_add(c - b'0')?;
        }
        let offset = control_string.len() - iter.len() - 1;
        (Some(code), &control_string[offset..])
    } else {
        (None, control_string)
    };
    let split_at = if rest.first()?.is_ascii_alphanumeric() {
        1
    } else {
        2
    };
    let (command, rest) = rest.split_at_checked(split_at)?;
    Some((code, command, rest))
}

fn process_osc(
    control_string: &Bytes,
    output: &mut BufferedOutput,
    input: &mut BufferedInput,
) -> Option<()> {
    let (code, text) = parse_osc(control_string)?;
    do_osc(code, text, output, input)
}

fn parse_osc(control_string: &Bytes) -> Option<(u8, ByteString)> {
    let mut iter = control_string.iter();
    let mut has_code = false;
    let mut code = 0u8;
    for &c in &mut iter {
        match c {
            b'0'..=b'9' => {
                has_code = true;
                code = code.checked_mul(10)?.checked_add(c - b'0')?;
            }
            b';' if has_code => break,
            _ => return None,
        }
    }

    let offset = control_string.len() - iter.as_slice().len();
    let text_bytes = control_string.slice(offset..);
    let text = ByteString::try_from(text_bytes).ok()?;
    Some((code, text))
}

fn do_osc(
    code: u8,
    text: ByteString,
    output: &mut BufferedOutput,
    input: &mut BufferedInput,
) -> Option<()> {
    match code {
        0 => {
            output.append(ControlFragment::SetIconLabel(text.clone()));
            output.append(ControlFragment::SetTitle(text));
        }
        1 => output.append(ControlFragment::SetIconLabel(text)),
        2 => output.append(ControlFragment::SetTitle(text)),
        3 => output.append(ControlFragment::SetXProperty(text)),
        4 => {
            let mut iter = text.split(';');
            iter.next()?;
            while let Some(color) = iter.next() {
                let spec = iter.next()?;
                let color_code = color.parse().ok()?;
                if spec == "?" {
                    let RgbColor { r, g, b } = output.get_xterm_color(color_code);
                    write!(input, "{color};rgb:{r}00/{g}00/{b}00");
                }
                if let Some(spec) = RgbColor::named(spec) {
                    output.set_xterm_color(color_code, spec);
                }
            }
        }
        10 => set_dynamic(DynamicColor::TextForeground, &text, output),
        11 => set_dynamic(DynamicColor::TextBackground, &text, output),
        12 => set_dynamic(DynamicColor::TextCursor, &text, output),
        13 => set_dynamic(DynamicColor::MouseForeground, &text, output),
        14 => set_dynamic(DynamicColor::MouseBackground, &text, output),
        15 => set_dynamic(DynamicColor::TektronixForeground, &text, output),
        16 => set_dynamic(DynamicColor::TektronixBackground, &text, output),
        17 => set_dynamic(DynamicColor::Highlight, &text, output),
        18 => set_dynamic(DynamicColor::TektronixCursor, &text, output),
        50 => output.append(ControlFragment::SetFont(text)),
        52 => {
            let &[selection, b';', ..] = (*text).as_bytes() else {
                return None;
            };
            let selection = SelectionData::from_code(selection)?;
            let (_, text) = text.split_at(2);
            output.append(ControlFragment::ManipulateSelection(selection, text));
        }
        _ => return None,
    }
    Some(())
}

fn set_dynamic(dynamic_color: DynamicColor, spec: &str, output: &mut BufferedOutput) {
    if let Some(spec) = RgbColor::named(spec) {
        output.append(ControlFragment::SetDynamicColor(dynamic_color, spec));
    }
}

fn decode_hex(sequence: &[u8], buf: &mut Vec<u8>) -> Option<()> {
    const fn hex_digit(byte: u8) -> u8 {
        if byte > b'9' {
            const TO_UPPERCASE_MASK: u8 = !0b0010_0000;
            (byte.wrapping_sub(b'A') & TO_UPPERCASE_MASK) + 10
        } else {
            byte.wrapping_sub(b'0')
        }
    }

    let (chunks, rest) = sequence.as_chunks();

    buf.reserve(sequence.len() / 2);

    for &[high, low] in chunks {
        let high = hex_digit(high);
        let low = hex_digit(low);
        if high >= 16 || low >= 16 {
            return None;
        }
        buf.push(high << 4 | low);
    }

    if rest.is_empty() { Some(()) } else { None }
}
