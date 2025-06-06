use std::str;

use bytes::BytesMut;
use flagset::FlagSet;

use super::color::TermColor;
use super::fragment::{
    EntityFragment, Output, OutputDrain, OutputFragment, TelnetFragment, TextFragment,
};
use super::shared_string::SharedString;
use super::span::{EntitySetter, SpanList, TextStyle};
use mxp::RgbColor;

macro_rules! debug_assert_utf8 {
    ($e:expr) => (debug_assert!(str::from_utf8(&$e).is_ok(), "`MudTransformer::receive_byte` failed to validate UTF8. This should NEVER happen.\nBytes: {:?}\nString: {}", &*$e, String::from_utf8_lossy(&$e)))
}

fn get_color(
    span_color: Option<TermColor>,
    ansi_color: TermColor,
    ignore_mxp: bool,
    default: TermColor,
) -> TermColor {
    match span_color {
        Some(span_color) if !ignore_mxp && span_color != default => span_color,
        _ => ansi_color,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BufferedOutput {
    buf: BytesMut,
    fragments: Vec<Output>,
    spans: SpanList,
    variables: mxp::EntityMap,

    in_line: bool,
    last_break: usize,
    last_linebreak: Option<usize>,

    ansi_flags: FlagSet<TextStyle>,
    ansi_foreground: TermColor,
    ansi_background: TermColor,
    colors: Vec<RgbColor>,
    ignore_mxp_colors: bool,

    in_variable: bool,
    variable: Vec<u8>,
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
            ansi_flags: FlagSet::default(),
            ansi_foreground: TermColor::WHITE,
            ansi_background: TermColor::BLACK,
            buf: BytesMut::new(),
            fragments: Vec::new(),
            ignore_mxp_colors: false,
            in_line: false,
            last_break: 0,
            last_linebreak: None,
            colors: Vec::new(),
            variables: mxp::EntityMap::new(),
            in_variable: false,
            variable: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.fragments.is_empty() && self.buf.is_empty()
    }

    pub fn set_colors(&mut self, colors: Vec<RgbColor>) {
        self.colors = colors;
    }

    fn color(&self, color: TermColor) -> RgbColor {
        match color {
            TermColor::Ansi(i) => match self.colors.get(usize::from(i)) {
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
        if self.in_line {
            let last_break = self.last_break;
            self.last_break = 0;
            self.fragments.drain(..last_break)
        } else {
            self.fragments.drain(..)
        }
    }

    pub fn append<T: Into<OutputFragment>>(&mut self, fragment: T) {
        // Reduce monomorphization
        fn inner(buffer: &mut BufferedOutput, fragment: OutputFragment) {
            if fragment.should_flush() {
                buffer.flush();
                if fragment.is_newline() {
                    buffer.in_line = false;
                }
            }
            if !fragment.is_visual() {
                if fragment == OutputFragment::Telnet(TelnetFragment::GoAhead) {
                    buffer.last_break = buffer.fragments.len() + 1;
                }
                buffer.fragments.push(Output::from(fragment));
                return;
            }
            if !buffer.in_line && !fragment.is_newline() {
                buffer.in_line = true;
                buffer.last_break = buffer.fragments.len();
            }
            let Some(span) = buffer.spans.get() else {
                buffer.fragments.push(Output::from(fragment));
                return;
            };
            buffer.fragments.push(Output {
                fragment,
                gag: span.gag,
                window: span.window.clone(),
            });
        }
        inner(self, fragment.into());
    }

    fn take_buf(&mut self) -> SharedString {
        let bytes = self.buf.split().freeze();
        debug_assert_utf8!(bytes);
        // SAFETY: MudTransformer::receive_byte sanitizes UTF8.
        unsafe { SharedString::from_utf8_unchecked(bytes) }
    }

    fn flush_last(&mut self, i: usize) {
        if self.buf.is_empty() {
            return;
        }
        let text = self.take_buf();
        if self.spans.len() < i {
            self.append(TextFragment {
                text,
                flags: self.ansi_flags,
                foreground: self.color(self.ansi_foreground),
                background: self.color(self.ansi_background),
                font: None,
                size: None,
                action: None,
                heading: None,
            });
            return;
        }
        let span = &self.spans[self.spans.len() - i];
        let ignore_colors = self.ignore_mxp_colors;
        self.append(TextFragment {
            flags: span.flags | self.ansi_flags,
            foreground: self.color(get_color(
                span.foreground,
                self.ansi_foreground,
                ignore_colors,
                TermColor::WHITE,
            )),
            background: self.color(get_color(
                span.background,
                self.ansi_background,
                ignore_colors,
                TermColor::BLACK,
            )),
            font: span.font.clone(),
            size: span.size,
            action: span.action.as_ref().map(|action| action.with_text(&text)),
            heading: span.heading,
            text,
        });
    }

    fn flush_mxp(&mut self) {
        self.flush_last(2);
    }

    pub fn flush(&mut self) {
        self.flush_last(1);
    }

    pub fn start_line(&mut self) {
        self.append(OutputFragment::LineBreak);
    }

    pub fn push(&mut self, byte: u8) {
        self.buf.extend_from_slice(&[byte]);
        self.spans.set_populated();
        if self.in_variable {
            self.variable.push(byte);
        }
    }

    pub fn append_text(&mut self, output: &str) {
        let output = output.as_bytes();
        self.buf.extend_from_slice(output);
        self.spans.set_populated();
        if self.in_variable {
            self.variable.extend_from_slice(output);
        }
    }

    pub fn append_utf8_char(&mut self, utf8: &[u8]) {
        if str::from_utf8(utf8).is_ok() {
            self.buf.extend_from_slice(utf8);
            if self.in_variable {
                self.variable.extend_from_slice(utf8);
            }
        } else {
            self.buf.extend_from_slice("�".as_bytes());
        }
        self.spans.set_populated();
    }

    pub fn append_subnegotiation(&mut self, code: u8, data: &[u8]) {
        self.flush();
        self.buf.extend_from_slice(data);
        let data = self.buf.split().freeze();
        self.append(TelnetFragment::Subnegotiation { code, data });
    }

    pub fn append_server_status(&mut self, key: &[u8], value: &[u8]) {
        self.flush();
        self.buf.extend_from_slice(key);
        let variable = self.buf.split().freeze();
        self.buf.extend_from_slice(value);
        let value = self.buf.split().freeze();
        self.append(TelnetFragment::ServerStatus { variable, value });
    }

    pub fn set_ansi_flag(&mut self, flag: TextStyle) {
        self.ansi_flags |= flag;
    }

    pub fn unset_ansi_flag(&mut self, flag: TextStyle) {
        self.ansi_flags -= flag;
    }

    pub fn set_ansi_foreground<C: Into<TermColor>>(&mut self, foreground: C) {
        let foreground = foreground.into();
        if self.ansi_foreground == foreground {
            return;
        }
        self.ansi_foreground = foreground;
        let Some(span) = self.spans.get() else {
            return;
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
        let Some(span) = self.spans.get() else {
            return;
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

    pub fn truncate_spans(&mut self, i: usize, entities: &mut mxp::EntityMap) {
        self.flush();
        let Some(entity) = self.spans.truncate(i) else {
            return;
        };
        self.in_variable = false;
        let variables = if entity.is_variable {
            &mut self.variables
        } else {
            &mut *entities
        };
        if entity.flags.contains(mxp::EntityKeyword::Delete) {
            self.variable.clear();
            variables.remove(&entity.name).ok();
            self.fragments.push(Output::from(EntityFragment::Unset {
                name: entity.name,
                is_variable: entity.is_variable,
            }));
            return;
        }
        debug_assert_utf8!(self.variable);
        // SAFETY: MudTransformer::receive_byte sanitizes UTF8.
        let text = unsafe { str::from_utf8_unchecked(&self.variable) };
        if let Ok(Some(entity)) = variables.set(&entity.name, text, None, entity.flags) {
            self.fragments
                .push(Output::from(EntityFragment::variable(&entity)));
        }
        self.variable.clear();
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

    pub fn set_mxp_font(&mut self, font: mxp::Font) {
        let mxp::Font {
            face,
            size,
            color,
            back,
        } = font;
        let mut changed = false;
        if let Some(face) = face {
            changed = self.spans.set_font(face) || changed;
        }
        if let Some(size) = size {
            changed = self.spans.set_size(size) || changed;
        }
        if let Some(color) = color {
            for fg in color.iter() {
                changed = match fg {
                    mxp::FontEffect::Color(fg) => self.spans.set_foreground(fg.into()),
                    mxp::FontEffect::Style(style) => self.spans.set_flag(style.into()),
                } || changed;
            }
        }
        if let Some(back) = back {
            changed = self.spans.set_background(back.into()) || changed;
        }
        if changed {
            self.flush_mxp();
        }
    }

    pub fn set_mxp_action(&mut self, action: mxp::Link) {
        if self.spans.set_action(action) {
            self.flush_mxp();
        }
    }

    pub fn set_mxp_heading(&mut self, heading: mxp::Heading) {
        if self.spans.set_heading(heading) {
            self.flush_mxp();
        }
    }

    pub fn set_mxp_entity(&mut self, entity: EntitySetter) {
        self.in_variable = true;
        if self.spans.set_entity(entity) {
            self.flush_mxp();
        }
    }

    pub fn set_mxp_gag(&mut self) {
        if self.spans.set_gag() {
            self.flush_mxp();
        }
    }

    pub fn set_mxp_window(&mut self, window: String) {
        if self.spans.set_window(window) {
            self.flush_mxp();
        }
    }

    pub fn published_variables(&self) -> mxp::PublishedIter {
        self.variables.published()
    }
}
