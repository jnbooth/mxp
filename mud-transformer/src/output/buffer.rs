use std::str;

use bytes::BytesMut;
use enumeration::EnumSet;

use crate::output::fragment::EffectFragment;

use super::color::TermColor;
use super::fragment::{Output, OutputDrain, OutputFragment, TelnetFragment, TextFragment};
use super::shared_string::SharedString;
use super::span::{InList, SpanList, TextFormat, TextStyle};
use mxp::RgbColor;

fn get_color(
    span_color: &Option<TermColor>,
    ansi_color: TermColor,
    ignore_mxp: bool,
    default: TermColor,
) -> TermColor {
    match span_color {
        Some(span_color) if !ignore_mxp && *span_color != default => *span_color,
        _ => ansi_color,
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BufferedOutput {
    spans: SpanList,
    ansi_flags: EnumSet<TextStyle>,
    ansi_foreground: TermColor,
    ansi_background: TermColor,
    buf: BytesMut,
    fragments: Vec<Output>,
    ignore_mxp_colors: bool,
    last_linebreak: Option<usize>,
    colors: Vec<RgbColor>,
}

impl Default for BufferedOutput {
    fn default() -> Self {
        Self::new()
    }
}

impl BufferedOutput {
    pub fn new() -> Self {
        Self {
            spans: SpanList::new(),
            ansi_flags: EnumSet::new(),
            ansi_foreground: TermColor::WHITE,
            ansi_background: TermColor::BLACK,
            buf: BytesMut::new(),
            fragments: Vec::new(),
            ignore_mxp_colors: false,
            last_linebreak: None,
            colors: Vec::new(),
        }
    }

    pub fn set_colors(&mut self, colors: Vec<RgbColor>) {
        self.colors = colors;
    }

    fn color(&self, color: TermColor) -> RgbColor {
        match color {
            TermColor::Ansi(i) => match self.colors.get(i as usize) {
                Some(color) => *color,
                None => RgbColor::xterm(i),
            },
            TermColor::Rgb(color) => color,
        }
    }

    pub fn last(&self) -> Option<u8> {
        self.buf.last().copied()
    }

    pub fn disable_mxp_colors(&mut self) {
        self.ignore_mxp_colors = true;
    }

    pub fn enable_mxp_colors(&mut self) {
        self.ignore_mxp_colors = false;
    }

    pub fn drain(&mut self) -> OutputDrain {
        self.last_linebreak = None;
        self.fragments.drain(..)
    }

    pub fn drain_complete(&mut self) -> OutputDrain {
        match self.last_linebreak.take() {
            Some(i) => self.fragments.drain(..=i),
            None => self.fragments.drain(self.fragments.len()..),
        }
    }

    fn output<T: Into<OutputFragment>>(&mut self, fragment: T) {
        let fragment = fragment.into();
        let output = match self.spans.get() {
            Some(span) => Output {
                fragment,
                gag: span.gag,
                window: span.window.clone(),
            },
            None => Output {
                fragment,
                gag: false,
                window: None,
            },
        };
        self.fragments.push(output)
    }

    fn take_buf(&mut self) -> SharedString {
        let bytes = self.buf.split().freeze();
        debug_assert!(str::from_utf8(&bytes).is_ok(), "`MudTransformer::receive_byte` failed to validate UTF8. This should NEVER happen.\nBytes: {:?}\nString: {}", &*bytes, String::from_utf8_lossy(&bytes));
        unsafe { SharedString::from_utf8_unchecked(bytes) }
    }

    fn flush_last(&mut self, i: usize) {
        if self.buf.is_empty() {
            return;
        }
        let text = self.take_buf();
        let fragment = if self.spans.len() < i {
            TextFragment {
                text,
                flags: self.ansi_flags,
                foreground: self.color(self.ansi_foreground),
                background: self.color(self.ansi_background),
                action: None,
                heading: None,
                variable: None,
            }
        } else {
            let span = &self.spans[self.spans.len() - i];
            let ignore_colors = self.ignore_mxp_colors;
            TextFragment {
                text,
                flags: span.flags | self.ansi_flags,
                foreground: self.color(get_color(
                    &span.foreground,
                    self.ansi_foreground,
                    ignore_colors,
                    TermColor::WHITE,
                )),
                background: self.color(get_color(
                    &span.background,
                    self.ansi_background,
                    ignore_colors,
                    TermColor::BLACK,
                )),
                action: span.action.clone().map(Box::new),
                heading: span.heading,
                variable: span.variable.clone(),
            }
        };
        self.output(fragment);
    }

    fn flush_mxp(&mut self) {
        self.flush_last(2);
    }

    pub fn flush(&mut self) {
        self.flush_last(1);
    }

    fn flush_line(&mut self) {
        self.flush();
        self.last_linebreak = Some(self.fragments.len());
    }

    pub fn start_line(&mut self) {
        self.flush_line();
        self.output(OutputFragment::LineBreak);
    }

    pub fn push(&mut self, byte: u8) {
        self.buf.extend_from_slice(&[byte]);
        self.spans.set_populated();
    }

    pub fn append(&mut self, output: &str) {
        self.buf.extend_from_slice(output.as_bytes());
        self.spans.set_populated();
    }

    pub fn append_utf8_char(&mut self, utf8: &[u8]) {
        if str::from_utf8(utf8).is_ok() {
            self.buf.extend_from_slice(utf8);
        } else {
            self.buf.extend_from_slice("�".as_bytes());
        }
        self.spans.set_populated();
    }

    pub fn append_afk(&mut self, challenge: &str) {
        self.flush();
        self.buf.extend_from_slice(challenge.as_bytes());
        let challenge = self.take_buf();
        self.output(TelnetFragment::Afk { challenge })
    }

    pub fn append_effect(&mut self, effect: EffectFragment) {
        self.flush();
        self.output(effect);
    }

    pub fn append_hr(&mut self) {
        self.flush_line();
        self.output(OutputFragment::Hr);
    }

    pub fn append_iac_ga(&mut self) {
        self.flush();
        self.output(TelnetFragment::IacGa);
    }

    pub fn append_image(&mut self, src: String) {
        self.flush();
        self.output(OutputFragment::Image(src));
    }

    pub fn append_mxp_error(&mut self, error: mxp::Error) {
        self.output(error);
    }

    pub fn append_page_break(&mut self) {
        self.flush_line();
        self.output(OutputFragment::PageBreak);
    }

    pub fn append_subnegotiation(&mut self, code: u8, data: &[u8]) {
        self.flush();
        self.buf.extend_from_slice(data);
        let data = self.buf.split().freeze();
        self.output(TelnetFragment::Subnegotiation { code, data })
    }

    pub fn append_telnet_naws(&mut self) {
        self.flush();
        self.output(TelnetFragment::Naws);
    }

    pub fn append_telnet_do(&mut self, code: u8) {
        self.flush();
        self.output(TelnetFragment::Do { code });
    }

    pub fn append_telnet_will(&mut self, code: u8) {
        self.flush();
        self.output(TelnetFragment::Will { code });
    }

    pub fn set_ansi_flag(&mut self, flag: TextStyle) {
        self.ansi_flags.insert(flag);
    }

    pub fn unset_ansi_flag(&mut self, flag: TextStyle) {
        self.ansi_flags.remove(flag);
    }

    pub fn set_ansi_foreground<C: Into<TermColor>>(&mut self, foreground: C) {
        let foreground = foreground.into();
        if self.ansi_foreground == foreground {
            return;
        }
        self.ansi_foreground = foreground;
        let span = match self.spans.get() {
            Some(span) => span,
            None => return,
        };
        match &span.foreground {
            Some(color) if *color == foreground => (),
            _ => self.flush(),
        }
    }

    pub fn set_ansi_background<C: Into<TermColor>>(&mut self, background: C) {
        let background = background.into();
        if self.ansi_background == background {
            return;
        }
        self.ansi_background = background;
        let span = match self.spans.get() {
            Some(span) => span,
            None => return,
        };
        match &span.background {
            Some(color) if *color == background => (),
            _ => self.flush(),
        }
        self.flush();
    }

    pub fn reset_ansi(&mut self) {
        self.flush();
        self.ansi_flags.clear();
        self.ansi_foreground = TermColor::WHITE;
        self.ansi_background = TermColor::BLACK;
    }

    pub fn reset_mxp(&mut self) {
        self.flush();
        self.spans.clear();
    }

    pub fn reset(&mut self) {
        self.reset_ansi();
        self.reset_mxp();
    }

    pub fn span_len(&self) -> usize {
        self.spans.len()
    }

    pub fn truncate_spans(&mut self, i: usize) {
        self.flush();
        self.spans.truncate(i);
    }

    pub fn format(&self) -> EnumSet<TextFormat> {
        self.spans.format()
    }

    pub fn set_format(&mut self, format: TextFormat) {
        self.spans.set_format(format);
    }

    pub fn unset_format(&mut self, format: TextFormat) {
        self.spans.unset_format(format);
    }

    pub fn set_mxp_flag(&mut self, flag: TextStyle) {
        if self.spans.set_flag(flag) && !self.ansi_flags.contains(flag) {
            self.flush_mxp();
        }
    }

    pub fn set_mxp_foreground<C: Into<TermColor>>(&mut self, foreground: C) {
        if self.spans.set_foreground(foreground.into()) {
            self.flush_mxp();
        }
    }

    pub fn set_mxp_background<C: Into<TermColor>>(&mut self, background: C) {
        if self.spans.set_background(background.into()) {
            self.flush_mxp();
        }
    }

    pub fn set_mxp_action(&mut self, action: mxp::Link) {
        if self.spans.set_action(action) {
            self.flush_mxp();
        }
    }

    pub fn set_mxp_list(&mut self, list: InList) {
        if self.spans.set_list(list) {
            self.flush_mxp();
        }
    }

    pub fn next_list_item(&mut self) -> Option<u32> {
        self.spans.next_list_item()
    }

    pub fn set_mxp_heading(&mut self, heading: mxp::Heading) {
        if self.spans.set_heading(heading) {
            self.flush_mxp();
        }
    }

    pub fn set_mxp_variable(&mut self, variable: String) {
        if self.spans.set_variable(variable) {
            self.flush_mxp();
        }
    }

    pub fn set_mxp_gag(&mut self) {
        if self.spans.set_gag() {
            self.flush_mxp()
        }
    }

    pub fn set_mxp_window(&mut self, window: String) {
        if self.spans.set_window(window) {
            self.flush_mxp()
        }
    }
}
