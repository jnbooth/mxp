use std::str;

use bytes_pool::{ByteString, BytesPool};
use flagset::FlagSet;
use mxp::RgbColor;

use super::color::TermColor;
use super::fragment::{
    EntityFragment, Output, OutputDrain, OutputFragment, TelnetFragment, TextFragment,
};
use super::span::{EntitySetter, SpanList, TextStyle};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct BufferedOutput {
    bytes_pool: BytesPool,
    text_buf: String,
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
    variable: String,
}

impl BufferedOutput {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.fragments.is_empty() && self.text_buf.is_empty()
    }

    pub fn set_colors(&mut self, colors: Vec<RgbColor>) {
        self.colors = colors;
    }

    fn color(&self, color: TermColor) -> Option<RgbColor> {
        match color {
            TermColor::Unset => None,
            TermColor::Ansi(i) => Some(match self.colors.get(usize::from(i)) {
                Some(color) => *color,
                None => RgbColor::xterm(i),
            }),
            TermColor::Rgb(color) => Some(color),
        }
    }

    pub fn last(&self) -> Option<u8> {
        self.text_buf.as_bytes().last().copied()
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

    pub fn append<T>(&mut self, fragment: T)
    where
        T: Into<OutputFragment>,
    {
        // Reduce monomorphization
        fn inner(buffer: &mut BufferedOutput, fragment: OutputFragment) {
            if fragment.should_flush() {
                buffer.flush();
                if fragment.is_newline() {
                    buffer.in_line = false;
                    buffer.ansi_flags.clear();
                    buffer.ansi_foreground = TermColor::Unset;
                    buffer.ansi_background = TermColor::Unset;
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

    fn take_buf(&mut self) -> ByteString {
        let buf = self.bytes_pool.share_str(&self.text_buf);
        self.text_buf.clear();
        buf
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
        let (foreground, background) = if self.ignore_mxp_colors {
            (self.ansi_foreground, self.ansi_background)
        } else {
            (span.foreground, span.background)
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
    pub fn append_text(&mut self, output: &str) {
        if self.text_buf.is_empty() {
            self.spans.set_populated();
        }
        self.text_buf.push_str(output);
        if self.in_variable {
            self.variable.push_str(output);
        }
    }

    pub fn append_char(&mut self, output: char) {
        if self.text_buf.is_empty() {
            self.spans.set_populated();
        }
        self.text_buf.push(output);
        if self.in_variable {
            self.variable.push(output);
        }
    }

    pub fn append_subnegotiation(&mut self, code: u8, data: &[u8]) {
        self.flush();
        let data = self.bytes_pool.share_bytes(data);
        self.append(TelnetFragment::Subnegotiation { code, data });
    }

    pub fn append_server_status(&mut self, key: &[u8], value: &[u8]) {
        self.flush();
        let variable = self.bytes_pool.share_bytes(key);
        let value = self.bytes_pool.share_bytes(value);
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
        match self.spans.get() {
            Some(span) if span.foreground == foreground => (),
            _ => self.flush(),
        }
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
            changed = self.spans.set_font(face) || changed;
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
        let window = self.bytes_pool.share_str(window);
        if self.spans.set_window(window) {
            self.flush_mxp();
        }
    }

    pub fn published_variables(&self) -> mxp::PublishedIter<'_> {
        self.variables.published()
    }
}
