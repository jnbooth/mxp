use bytes::BytesMut;
use enumeration::EnumSet;

use super::fragment::{OutputDrain, OutputFragment, TextFragment};
use super::output::Output;
use super::span::{Heading, InList, SpanList, TextFormat, TextStyle};
use mxp::WorldColor;

fn get_color(
    span_color: &Option<WorldColor>,
    ansi_color: WorldColor,
    ignore_mxp: bool,
) -> WorldColor {
    match span_color {
        Some(span_color) if !ignore_mxp => *span_color,
        _ => ansi_color,
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BufferedOutput {
    spans: SpanList,
    ansi_flags: EnumSet<TextStyle>,
    ansi_foreground: WorldColor,
    ansi_background: WorldColor,
    buf: BytesMut,
    fragments: Vec<OutputFragment>,
    ignore_mxp_colors: bool,
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
            ansi_foreground: WorldColor::WHITE,
            ansi_background: WorldColor::BLACK,
            buf: BytesMut::new(),
            fragments: Vec::new(),
            ignore_mxp_colors: false,
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
        self.fragments.drain(..)
    }

    pub fn drain_complete(&mut self) -> OutputDrain {
        self.fragments.drain(..self.fragments.len() - 1)
    }

    fn flush_last(&mut self, i: usize) {
        if self.buf.is_empty() {
            return;
        }
        let text = self.buf.split().freeze();
        let fragment = if self.spans.len() < i {
            TextFragment {
                text,
                flags: self.ansi_flags,
                foreground: self.ansi_foreground,
                background: self.ansi_background,
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
                foreground: get_color(&span.foreground, self.ansi_foreground, ignore_colors),
                background: get_color(&span.background, self.ansi_background, ignore_colors),
                action: span.action.clone(),
                heading: span.heading,
                variable: span.variable.clone(),
            }
        };
        self.fragments.push(OutputFragment::Text(fragment));
    }

    fn flush_mxp(&mut self) {
        self.flush_last(2);
    }

    fn flush(&mut self) {
        self.flush_last(1);
    }

    pub fn start_line(&mut self) {
        self.buf.extend_from_slice(b"\n");
        self.flush();
    }

    pub fn append<O: Output>(&mut self, output: O) {
        output.output(&mut self.buf);
        self.spans.set_populated();
    }

    pub fn append_hr(&mut self) {
        self.flush();
        self.fragments.push(OutputFragment::Hr);
    }

    pub fn append_image(&mut self, src: String) {
        self.flush();
        self.fragments.push(OutputFragment::Image(src));
    }

    pub fn set_ansi_flag(&mut self, flag: TextStyle) {
        self.ansi_flags.insert(flag);
    }

    pub fn unset_ansi_flag(&mut self, flag: TextStyle) {
        self.ansi_flags.remove(flag);
    }

    pub fn set_ansi_foreground<C: Into<WorldColor>>(&mut self, foreground: C) {
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

    pub fn set_ansi_background<C: Into<WorldColor>>(&mut self, background: C) {
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
        self.ansi_foreground = WorldColor::WHITE;
        self.ansi_background = WorldColor::BLACK;
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

    pub fn set_mxp_foreground<C: Into<WorldColor>>(&mut self, foreground: C) {
        if self.spans.set_foreground(foreground.into()) {
            self.flush_mxp();
        }
    }

    pub fn set_mxp_background<C: Into<WorldColor>>(&mut self, background: C) {
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

    pub fn set_mxp_heading(&mut self, heading: Heading) {
        if self.spans.set_heading(heading) {
            self.flush_mxp();
        }
    }

    pub fn set_mxp_variable(&mut self, variable: String) {
        if self.spans.set_variable(variable) {
            self.flush_mxp();
        }
    }
}
