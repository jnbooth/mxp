use std::str;

use bytes::BytesMut;
use bytestring::ByteString;
use flagset::FlagSet;
use mxp::RgbColor;

use super::fragment::{
    ControlFragment, EntityFragment, Output, OutputDrain, OutputFragment, TelnetFragment,
    TextFragment,
};
use super::span::{EntitySetter, SpanList, TextStyle};
use crate::responses::SgrReport;
use crate::term::{TermColor, XTermPalette};

fn last_printable_char(s: &str) -> Option<char> {
    s.chars()
        .rev()
        .find(|c| !matches!(c, '\0'..'\x20' | '\x7f'..='\u{9f}'))
}

#[derive(Clone, Debug, Default)]
pub(crate) struct BufferedOutput {
    text_buf: BytesMut,
    fragments: Vec<Output>,
    spans: SpanList,
    variables: mxp::EntityMap,

    in_line: bool,
    last_break: usize,
    last_linebreak: Option<usize>,
    last_char: Option<char>,

    ansi_flags: FlagSet<TextStyle>,
    ansi_foreground: TermColor,
    ansi_background: TermColor,
    xterm_palette: Box<XTermPalette>,
    ignore_mxp_colors: bool,

    cursor: usize,
    in_variable: bool,
    variable: String,
}

impl BufferedOutput {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.fragments.is_empty() && self.text_buf.is_empty()
    }

    pub fn set_colors(&mut self, colors: &[RgbColor]) {
        self.xterm_palette.set_defaults(colors);
    }

    pub fn get_xterm_color(&self, code: u8) -> RgbColor {
        self.xterm_palette[code]
    }

    pub fn set_xterm_color(&mut self, code: u8, color: RgbColor) {
        self.xterm_palette[code] = color;
    }

    pub fn reset_xterm_color(&mut self, code: u8) {
        self.xterm_palette.reset_color(code);
    }

    pub fn ansi_mode(&self) -> SgrReport {
        SgrReport {
            flags: self.ansi_flags,
            foreground: self.ansi_foreground,
            background: self.ansi_background,
        }
    }

    fn color(&self, color: TermColor) -> Option<RgbColor> {
        match color {
            TermColor::Unset => None,
            TermColor::Ansi(i) => Some(self.xterm_palette[i]),
            TermColor::Rgb(color) => Some(color),
        }
    }

    pub fn disable_mxp_colors(&mut self) {
        self.ignore_mxp_colors = true;
    }

    pub fn enable_mxp_colors(&mut self) {
        self.ignore_mxp_colors = false;
    }

    pub fn drain(&mut self) -> OutputDrain<'_> {
        self.last_linebreak = None;
        self.fragments.drain(..)
    }

    pub fn drain_complete(&mut self) -> OutputDrain<'_> {
        if self.in_line {
            let last_break = self.last_break;
            self.last_break = 0;
            self.fragments.drain(..last_break)
        } else {
            self.fragments.drain(..)
        }
    }

    pub fn append_repeated(&mut self, text: &str, times: usize) {
        if times == 0 {
            return;
        }
        if self.text_buf.is_empty() {
            self.spans.set_populated();
        }
        for _ in 0..times {
            self.text_buf.extend_from_slice(text.as_bytes());
            if self.in_variable {
                self.variable.push_str(text);
            }
        }
        self.cursor += text.len() * times;
        if let Some(c) = last_printable_char(text) {
            self.last_char = Some(c);
        }
    }

    pub fn last_printed_character(&self) -> Option<char> {
        if let Some(c) = last_printable_char(self.view_buf()) {
            return Some(c);
        }
        self.last_char
    }

    #[inline]
    pub fn append<T>(&mut self, fragment: T)
    where
        T: Into<OutputFragment>,
    {
        self.append_fragment(fragment.into());
    }

    // Reduce monomorphization.
    fn append_fragment(&mut self, fragment: OutputFragment) {
        if !self.text_buf.is_empty() && fragment.should_flush() {
            self.flush();
        }
        if fragment.resets_line() {
            self.cursor = 0;
            self.in_line = false;
        } else if !self.in_line && fragment.is_line_content() {
            self.in_line = true;
            self.last_break = self.fragments.len();
        }
        match &fragment {
            OutputFragment::LineBreak => {
                self.reset_ansi_after_flush();
            }
            OutputFragment::Control(ControlFragment::ResetTerminal { .. }) => {
                self.xterm_palette.reset();
                self.reset_ansi_after_flush();
            }
            OutputFragment::Text(fragment) => {
                self.cursor += fragment.text.len();
                if let Some(c) = last_printable_char(&fragment.text) {
                    self.last_char = Some(c);
                }
            }
            OutputFragment::Telnet(TelnetFragment::GoAhead) => {
                self.last_break = self.fragments.len() + 1;
            }
            _ => (),
        }
        if fragment.is_windowless() {
            self.fragments.push(fragment.into());
            return;
        }
        let Some(span) = self.spans.get() else {
            self.fragments.push(fragment.into());
            return;
        };
        self.fragments.push(Output {
            fragment,
            gag: span.gag,
            window: span.window.clone(),
        });
    }

    pub fn append_tab(&mut self) {
        const TABS: &[u8] = b"        ";
        let text_cursor = self.cursor + self.text_buf.len();
        let spacing = text_cursor % TABS.len();
        let spaces = if spacing == 0 { TABS } else { &TABS[..spacing] };
        self.text_buf.extend_from_slice(spaces);
    }

    fn take_buf(&mut self) -> ByteString {
        let buf = self.text_buf.split().freeze();
        // SAFETY: `self.text_buf` contains only valid UTF-8.
        unsafe { ByteString::from_bytes_unchecked(buf) }
    }

    fn view_buf(&self) -> &str {
        // SAFETY: `self.text_buf` contains only valid UTF-8.
        unsafe { str::from_utf8_unchecked(&self.text_buf) }
    }

    fn flush_last(&mut self, i: usize) {
        if self.text_buf.is_empty() {
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
        let foreground = if self.ignore_mxp_colors || span.foreground == TermColor::Unset {
            self.ansi_foreground
        } else {
            span.foreground
        };
        let background = if self.ignore_mxp_colors || span.background == TermColor::Unset {
            self.ansi_background
        } else {
            span.background
        };
        self.append(TextFragment {
            flags: span.flags | self.ansi_flags,
            foreground: self.color(foreground),
            background: self.color(background),
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

    #[inline]
    pub fn append_char(&mut self, output: char) {
        self.append_text(output.encode_utf8(&mut [0; 4]));
    }

    pub fn append_text(&mut self, output: &str) {
        if self.text_buf.is_empty() {
            self.spans.set_populated();
        }
        self.text_buf.extend_from_slice(output.as_bytes());
        if self.in_variable {
            self.variable.push_str(output);
        }
    }

    pub fn set_ansi_flag(&mut self, flag: TextStyle) {
        self.flush();
        self.ansi_flags |= flag;
    }

    pub fn unset_ansi_flag(&mut self, flag: TextStyle) {
        self.flush();
        self.ansi_flags -= flag;
    }

    pub fn set_ansi_foreground<C: Into<TermColor>>(&mut self, foreground: C) {
        let foreground = foreground.into();
        if self.ansi_foreground == foreground {
            return;
        }
        self.flush();
        self.ansi_foreground = foreground;
    }

    pub fn set_ansi_background<C: Into<TermColor>>(&mut self, background: C) {
        let background = background.into();
        if self.ansi_background == background {
            return;
        }
        match self.spans.get() {
            Some(span) if span.background == background => (),
            _ => self.flush(),
        }
        self.ansi_background = background;
    }

    pub fn reset_ansi(&mut self) {
        self.flush();
        self.reset_ansi_after_flush();
    }

    fn reset_ansi_after_flush(&mut self) {
        self.spans.reset_ansi();
        self.ansi_flags.clear();
        self.ansi_foreground = TermColor::Unset;
        self.ansi_background = TermColor::Unset;
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
        if let Ok(Some(entity)) = variables.set(&entity.name, &self.variable, None, entity.flags) {
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
            changed = self.spans.set_font(&face) || changed;
        }
        if let Some(size) = size {
            changed = self.spans.set_size(size) || changed;
        }
        if let Some(color) = color {
            for fg in &color {
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

    pub fn set_mxp_window(&mut self, window: &str) {
        if self.spans.set_window(window) {
            self.flush_mxp();
        }
    }

    pub fn published_variables(&self) -> mxp::PublishedIter<'_> {
        self.variables.published()
    }
}
