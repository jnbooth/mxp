use std::{fmt, str};

use bytestring::ByteString;
use bytestringmut::ByteStringMut;
use flagset::FlagSet;
use mxp::RgbColor;

use super::register::OutputRegister;
use super::span::{Span, SpanList};
use super::{
    ControlFragment, Link, Output, OutputDrain, OutputFragment, TelnetFragment, TextFragment,
    TextStyle,
};
use crate::responses::SgrReport;
use crate::term::{TermColor, XTermPalette};

fn last_printable_char(s: &str) -> Option<char> {
    s.chars()
        .rev()
        .find(|c| !matches!(c, '\0'..'\x20' | '\x7f'..='\u{9f}'))
}

#[derive(Clone, Debug, Default)]
pub(crate) struct BufferedOutput {
    text_buf: ByteStringMut,
    fragments: Vec<Output>,
    spans: SpanList,
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
    active_registers: usize,
    registers: Vec<OutputRegister>,
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

    pub fn xterm_colors(&self) -> &[RgbColor; 256] {
        self.xterm_palette.as_slice()
    }

    pub fn xterm_colors_mut(&mut self) -> &mut [RgbColor; 256] {
        self.xterm_palette.as_mut_slice()
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

    pub fn into_output(mut self) -> Vec<Output> {
        self.flush();
        self.fragments
    }

    pub fn append_repeated(&mut self, text: &str, times: usize) {
        if times == 0 {
            return;
        }
        for _ in 0..times {
            self.text_buf.push_str(text);
        }
        for register in &mut self.registers[..self.active_registers] {
            for _ in 0..times {
                register.push_str(text);
            }
        }
        if let Some(c) = last_printable_char(text) {
            self.last_char = Some(c);
        }
    }

    pub fn last_printed_character(&self) -> Option<char> {
        last_printable_char(&self.text_buf).or(self.last_char)
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
        const TABS: &[&str; 8] = &[
            "        ", " ", "  ", "   ", "    ", "     ", "      ", "       ",
        ];
        let text_cursor = self.cursor + self.text_buf.len();
        let spacing = text_cursor & 7;
        self.text_buf.push_str(TABS[spacing]);
    }

    fn take_buf(&mut self) -> ByteString {
        self.text_buf.split().freeze()
    }

    fn start_register(&mut self) -> usize {
        if self.active_registers == self.registers.len() {
            self.registers.push(OutputRegister::new());
        }
        let index = self.active_registers;
        self.active_registers += 1;
        index
    }

    fn flush_with(&self, text: ByteString, span: &Span) -> TextFragment {
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
        TextFragment {
            flags: span.flags | self.ansi_flags,
            foreground: self.color(foreground),
            background: self.color(background),
            font: span.font.clone(),
            size: span.size,
            link: span.link.as_ref().map(|link| link.for_text(&text)),
            heading: span.heading,
            text,
        }
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
                link: None,
                heading: None,
            });
            return;
        }
        let span = &self.spans[self.spans.len() - i];
        self.append(self.flush_with(text, span));
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
        self.text_buf.push(output);
        for register in &mut self.registers[..self.active_registers] {
            register.push(output);
        }
    }

    pub fn append_text(&mut self, output: &str) {
        self.text_buf.push_str(output);
        for register in &mut self.registers[..self.active_registers] {
            register.push_str(output);
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
        self.reset_ansi();
        self.spans.clear();
    }

    pub fn span_len(&self) -> usize {
        self.spans.len()
    }

    pub fn truncate_spans(&mut self, i: usize, mxp_state: &mut mxp::State) {
        let Some(mut span) = self.spans.truncate(i) else {
            return;
        };
        let active_span = self.spans.get();
        if self.active_registers == 0 {
            if !self.text_buf.is_empty() {
                let text = self.take_buf();
                self.append(self.flush_with(text, &span));
            }
            return;
        }
        let highest_active_register = match active_span {
            Some(span) => span
                .entity
                .unwrap_or_default()
                .max(span.parse_as.unwrap_or_default())
                .max(span.variable.unwrap_or_default()),
            None => 0,
        };
        for register in self.registers[highest_active_register..self.active_registers]
            .iter_mut()
            .rev()
        {
            if let Some(output) = register.finalize(mxp_state) {
                self.fragments.push(output.into());
            }
        }
        self.active_registers = highest_active_register;
        if self.text_buf.is_empty() {
            return;
        }
        if let Some(active_span) = active_span {
            span.entity = active_span.entity;
            span.parse_as = active_span.parse_as;
            span.variable = active_span.variable;
            if span == *active_span {
                return;
            }
        } else if span == Span::default() {
            return;
        }
        let text = self.take_buf();
        self.append(self.flush_with(text, &span));
    }

    pub fn set_mxp_flag(&mut self, flag: TextStyle) {
        if self.spans.set_flag(flag, self.text_buf.is_empty()) && !self.ansi_flags.contains(flag) {
            self.flush_mxp();
        }
    }

    pub fn set_mxp_color(&mut self, color: mxp::Color) {
        let empty = self.text_buf.is_empty();
        let set_foreground = if let Some(fg) = color.fore {
            self.spans.set_foreground(fg.into(), empty)
        } else {
            false
        };
        let set_background = if let Some(bg) = color.back {
            self.spans.set_background(bg.into(), empty)
        } else {
            false
        };
        if set_foreground || set_background {
            self.flush_mxp();
        }
    }

    pub fn set_mxp_foreground<C: Into<TermColor>>(&mut self, foreground: C) {
        let empty = self.text_buf.is_empty();
        if self.spans.set_foreground(foreground.into(), empty) {
            self.flush_mxp();
        }
    }

    pub fn set_mxp_background<C: Into<TermColor>>(&mut self, background: C) {
        let empty = self.text_buf.is_empty();
        if self.spans.set_background(background.into(), empty) {
            self.flush_mxp();
        }
    }

    pub fn set_mxp_font<S: AsRef<str>>(&mut self, font: mxp::Font<S>) {
        let mxp::Font {
            face,
            size,
            color,
            style,
            back,
        } = font;
        let mut changed = false;
        let empty = self.text_buf.is_empty();
        if let Some(face) = face {
            changed = self.spans.set_font(face.as_ref(), empty) || changed;
        }
        if let Some(size) = size {
            changed = self.spans.set_size(size, empty) || changed;
        }
        if let Some(color) = color {
            changed = self.spans.set_foreground(color.into(), empty) || changed;
        }
        for flag in style {
            changed = self.spans.set_flag(flag.into(), empty) || changed;
        }
        if let Some(back) = back {
            changed = self.spans.set_background(back.into(), empty) || changed;
        }
        if changed {
            self.flush_mxp();
        }
    }

    pub fn set_mxp_line_tag(&mut self, tag: &mxp::LineTagProperties) {
        if tag.gag {
            self.set_mxp_gag();
        }
        if let Some(window) = &tag.window {
            self.set_mxp_window(window.into());
        }
        if let Some(fore) = tag.fore {
            self.set_mxp_foreground(fore);
        }
        if let Some(back) = tag.back {
            self.set_mxp_background(back);
        }
    }

    pub fn set_mxp_link<T: Into<Link>>(&mut self, link: T) {
        if self.spans.set_link(link.into(), self.text_buf.is_empty()) {
            self.flush_mxp();
        }
    }

    pub fn set_mxp_heading(&mut self, heading: mxp::Heading) {
        if self.spans.set_heading(heading, self.text_buf.is_empty()) {
            self.flush_mxp();
        }
    }

    pub fn set_mxp_entity<S: AsRef<str>>(&mut self, var: mxp::Var<S>) {
        let register = self.start_register();
        self.registers[register].set_entity(var);
        if self
            .spans
            .set_entity(register + 1, self.text_buf.is_empty())
        {
            self.flush_mxp();
        }
    }

    pub fn set_mxp_variable(&mut self, var: &str) {
        let register = self.start_register();
        self.registers[register].set_variable(var);
        if self
            .spans
            .set_variable(register + 1, self.text_buf.is_empty())
        {
            self.flush_mxp();
        }
    }

    pub fn set_mxp_gag(&mut self) {
        if self.spans.set_gag(self.text_buf.is_empty()) {
            self.flush_mxp();
        }
    }

    pub fn set_mxp_parse_as(&mut self, parse_as: mxp::ParseAs) {
        let register = self.start_register();
        self.registers[register].set_parse_as(parse_as);
        if self
            .spans
            .set_parse_as(register + 1, self.text_buf.is_empty())
        {
            self.flush_mxp();
        }
    }

    pub fn set_mxp_window<S: AsRef<str>>(&mut self, window: mxp::Dest<S>) {
        if self.spans.set_window(window, self.text_buf.is_empty()) {
            self.flush_mxp();
        }
    }

    #[inline]
    pub fn write_fmt(&mut self, args: fmt::Arguments) {
        fmt::Write::write_fmt(self, args).unwrap();
    }
}

impl fmt::Write for BufferedOutput {
    #[inline]
    fn write_char(&mut self, c: char) -> fmt::Result {
        self.append_char(c);
        Ok(())
    }

    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.append_text(s);
        Ok(())
    }
}
