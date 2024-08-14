use std::borrow::Cow;
use std::num::NonZeroU8;
use std::{io, mem};

use super::config::{TransformerConfig, UseMxp};
use super::input::{BufferedInput, Drain as InputDrain};
use super::phase::Phase;
use super::tag::{Tag, TagList};
use crate::output::{BufferedOutput, InList, OutputDrain, TermColor, TextFormat, TextStyle};
use crate::receive::{Decompress, ReceiveCursor};
use crate::EffectFragment;
use enumeration::EnumSet;
use mxp::escape::{ansi, telnet};
use mxp::RgbColor;

fn input_mxp_auth(input: &mut BufferedInput, auth: &str) {
    if auth.is_empty() {
        return;
    }
    input.append(auth.as_bytes());
    input.append(b"\r\n");
}

#[derive(Debug)]
pub struct Transformer {
    config: TransformerConfig,
    decompressing: bool,
    decompress: Decompress,

    phase: Phase,

    mxp_active: bool,
    pueblo_active: bool,
    supports_mccp_2: bool,
    no_echo: bool,

    mxp_script: bool,
    suppress_newline: bool,
    mxp_mode_default: mxp::Mode,
    mxp_mode: mxp::Mode,
    mxp_mode_previous: mxp::Mode,
    mxp_quote_terminator: Option<NonZeroU8>,
    mxp_string: Vec<u8>,
    mxp_active_tags: TagList,
    mxp_state: mxp::State,

    subnegotiation_type: u8,
    subnegotiation_data: Vec<u8>,
    ttype_sequence: u8,
    ansi_code: u8,
    ansi_red: u8,
    ansi_green: u8,
    ansi_blue: u8,

    utf8_sequence: Vec<u8>,
    output: BufferedOutput,
    input: BufferedInput,
}

impl Default for Transformer {
    fn default() -> Self {
        Self::new(TransformerConfig::default())
    }
}

impl Transformer {
    pub fn new(config: TransformerConfig) -> Self {
        let mut output = BufferedOutput::new();
        output.set_colors(config.colors.clone());
        if config.ignore_mxp_colors {
            output.disable_mxp_colors();
        }
        Self {
            phase: Phase::Normal,
            decompressing: false,
            decompress: Decompress::new(),

            mxp_active: config.use_mxp == UseMxp::Always,
            pueblo_active: false,
            supports_mccp_2: false,
            no_echo: false,

            mxp_script: false,
            suppress_newline: false,
            mxp_mode_default: mxp::Mode::OPEN,
            mxp_mode: mxp::Mode::OPEN,
            mxp_mode_previous: mxp::Mode::OPEN,
            mxp_quote_terminator: None,
            mxp_string: Vec::new(),
            mxp_active_tags: TagList::new(),
            mxp_state: mxp::State::new(),

            subnegotiation_type: 0,
            subnegotiation_data: Vec::new(),
            ttype_sequence: 0,
            ansi_code: 0,
            ansi_red: 0,
            ansi_green: 0,
            ansi_blue: 0,

            utf8_sequence: Vec::with_capacity(4),
            output,
            input: BufferedInput::new(),

            config,
        }
    }

    pub fn set_config(&mut self, config: TransformerConfig) {
        if config.ignore_mxp_colors {
            self.output.disable_mxp_colors();
        } else {
            self.output.enable_mxp_colors();
        }
        if config.colors != self.config.colors {
            self.output.set_colors(config.colors.clone());
        }
        match config.use_mxp {
            UseMxp::Always => self.mxp_on(false, false),
            UseMxp::Never => self.mxp_off(true),
            UseMxp::Command | UseMxp::Query => (),
        }
        self.config = config;
    }

    pub fn drain_output(&mut self) -> OutputDrain {
        self.output.drain_complete()
    }

    pub fn flush_output(&mut self) -> OutputDrain {
        self.output.flush();
        self.output.drain()
    }

    pub fn drain_input(&mut self) -> Option<InputDrain> {
        self.input.drain()
    }

    pub fn published_entities(&self) -> mxp::PublishedIter {
        self.mxp_state.published_entities()
    }

    pub fn published_variables(&self) -> mxp::PublishedIter {
        self.output.published_variables()
    }

    fn handle_mxp_error(&mut self, err: mxp::Error) {
        self.output.append_mxp_error(err);
    }

    fn take_mxp_string(&mut self) -> mxp::Result<String> {
        String::from_utf8(mem::take(&mut self.mxp_string)).map_err(|e| {
            let bytes_debug = format!("{:?}", e.as_bytes());
            mxp::Error::new(bytes_debug, mxp::ErrorKind::MalformedBytes)
        })
    }

    fn mxp_restore_mode(&mut self) {
        if self.mxp_mode == mxp::Mode::SECURE_ONCE {
            self.mxp_mode = self.mxp_mode_previous;
        }
    }

    fn mxp_off(&mut self, completely: bool) {
        self.output.reset();

        if !self.mxp_active {
            return;
        }

        let closed = self.mxp_active_tags.last_resettable_index();
        self.mxp_close_tags_from(closed);
        self.mxp_script = false;

        if !completely {
            return;
        }
        self.mxp_mode_change(Some(mxp::Mode::OPEN));
        if self.phase.is_mxp() {
            self.phase = Phase::Normal;
        }
        self.pueblo_active = false;
        self.mxp_active = false;
    }

    fn mxp_on(&mut self, pueblo: bool, manual: bool) {
        if self.mxp_active {
            return;
        }

        self.mxp_active = true;
        self.pueblo_active = pueblo;
        self.mxp_script = false;

        if manual {
            return;
        }

        self.mxp_mode_default = mxp::Mode::OPEN;
        self.mxp_mode = mxp::Mode::OPEN;
        self.mxp_active_tags.clear();
        self.mxp_state.clear();
    }

    fn mxp_endtag(&mut self, tag_body: &str) -> mxp::Result<()> {
        let was_secure = self.mxp_mode.is_secure();
        self.mxp_restore_mode();
        let name = Tag::parse_closing_tag(tag_body)?;
        let (closed, _tag) = self.mxp_active_tags.find_last(was_secure, name)?;
        self.mxp_close_tags_from(closed);
        Ok(())
    }

    fn mxp_definition(&mut self, tag: &str) -> mxp::Result<()> {
        let was_secure = self.mxp_mode.is_secure();
        self.mxp_restore_mode();
        if !was_secure {
            return Err(mxp::Error::new(
                tag,
                mxp::ErrorKind::DefinitionWhenNotSecure,
            ));
        }
        self.mxp_state.define(tag)
    }

    fn mxp_collected_element(&mut self) -> mxp::Result<()> {
        match mxp::Element::collect(&self.take_mxp_string()?)? {
            mxp::CollectedElement::Definition(text) => self.mxp_definition(text),
            mxp::CollectedElement::TagClose(text) => self.mxp_endtag(text),
            mxp::CollectedElement::TagOpen(text) => {
                let mxp_state = mem::take(&mut self.mxp_state);
                let result = self.mxp_start_tag(text, &mxp_state);
                self.mxp_state = mxp_state;
                result
            }
        }
    }

    fn mxp_start_tag(&mut self, tag: &str, mxp_state: &mxp::State) -> mxp::Result<()> {
        let secure = self.mxp_mode.is_secure();
        self.mxp_restore_mode();
        let mut words = mxp::Words::new(tag);
        let name = words.validate_next_or(mxp::ErrorKind::InvalidElementName)?;
        let component = mxp_state.get_component(name)?;
        let tag = Tag::new(component, secure, self.output.span_len())?;
        self.mxp_active_tags.push(tag);

        if let Some(variable) = component.variable() {
            self.output.set_mxp_variable(variable, EnumSet::new());
        }

        let mut args = mxp::Arguments::parse(words)?;

        match component {
            mxp::ElementComponent::Atom(atom) => {
                let scanner = mxp_state.decode_args(&mut args);
                self.mxp_open_atom(mxp::Action::new(atom.action, scanner)?);
            }
            mxp::ElementComponent::Custom(el) => self.mxp_open_element(el, &args, mxp_state)?,
        }

        Ok(())
    }

    fn mxp_open_element(
        &mut self,
        el: &mxp::Element,
        args: &mxp::Arguments,
        mxp_state: &mxp::State,
    ) -> mxp::Result<()> {
        if el.gag {
            self.output.set_mxp_gag();
        }
        if let Some(window) = &el.window {
            self.output.set_mxp_window(window.clone());
        }
        for action in mxp_state.decode_element(el, args) {
            self.mxp_open_atom(action?);
        }
        if let Some(fore) = el.fore {
            self.output.set_mxp_foreground(fore);
        }
        if let Some(back) = el.back {
            self.output.set_mxp_background(back);
        }
        Ok(())
    }

    fn mxp_open_atom(&mut self, action: mxp::Action<Cow<str>>) {
        use mxp::{Action, MxpKeyword};
        /*
        if action == Action::Hyperlink && args.get("xch_cmd").is_some() {
            self.pueblo_active = true;
            action = Action::Send;
        }*/
        match action {
            Action::Heading(heading) => self.output.set_mxp_heading(heading),
            Action::Bold => self.output.set_mxp_flag(TextStyle::Bold),
            Action::Underline => self.output.set_mxp_flag(TextStyle::Underline),
            Action::Italic => self.output.set_mxp_flag(TextStyle::Italic),
            Action::Color { fore, back } => {
                if let Some(fg) = fore {
                    self.output.set_mxp_foreground(fg);
                }
                if let Some(bg) = back {
                    self.output.set_mxp_background(bg);
                }
            }
            Action::High => self.output.set_mxp_flag(TextStyle::Highlight),
            Action::Link(link) => {
                self.output.set_mxp_action(link.clone());
            }
            Action::Font {
                face,
                size,
                fgcolor,
                bgcolor,
            } => {
                if let Some(face) = face {
                    self.output.set_mxp_font(face.into_owned());
                }
                if let Some(size) = size {
                    self.output.set_mxp_size(size);
                }
                if let Some(fgcolor) = fgcolor {
                    for fg in fgcolor.iter() {
                        match fg {
                            mxp::FontEffect::Color(fg) => self.output.set_mxp_foreground(fg),
                            mxp::FontEffect::Style(style) => self.output.set_mxp_flag(style.into()),
                        }
                    }
                }
                if let Some(bg) = bgcolor {
                    self.output.set_mxp_background(bg);
                }
            }
            Action::Version => self.input.append(
                mxp::responses::identify(&self.config.app_name, &self.config.version).as_bytes(),
            ),
            Action::Afk { challenge } => {
                let challenge = challenge.as_deref().unwrap_or_default();
                self.output.append_afk(challenge);
            }
            Action::Support { supported } => self.input.append(&supported),
            Action::User => input_mxp_auth(&mut self.input, &self.config.player),
            Action::Password => input_mxp_auth(&mut self.input, &self.config.password),
            Action::Br => {
                self.output.start_line();
            }
            Action::SBr => {
                self.output.push(b' ');
            }
            Action::Reset => {
                self.mxp_off(false);
            }
            Action::Mxp { keywords } => {
                if keywords.contains(MxpKeyword::Off) {
                    self.mxp_off(true);
                }

                if keywords.contains(MxpKeyword::DefaultLocked) {
                    self.mxp_mode_default = mxp::Mode::LOCKED;
                } else if keywords.contains(MxpKeyword::DefaultSecure) {
                    self.mxp_mode_default = mxp::Mode::SECURE;
                } else if keywords.contains(MxpKeyword::DefaultOpen) {
                    self.mxp_mode_default = mxp::Mode::OPEN;
                }

                if keywords.contains(MxpKeyword::IgnoreNewlines) {
                    self.output.set_format(TextFormat::Paragraph);
                } else if keywords.contains(MxpKeyword::UseNewlines) {
                    self.output.unset_format(TextFormat::Paragraph);
                }
            }
            Action::P => self.output.set_format(TextFormat::Paragraph),
            Action::Script => self.mxp_script = true,
            Action::Hr => self.output.append_hr(),
            Action::Pre => self.output.set_format(TextFormat::Pre),
            Action::Ul => self.output.set_mxp_list(InList::Unordered),
            Action::Ol => self.output.set_mxp_list(InList::Ordered(0)),
            Action::Li => match self.output.next_list_item() {
                Some(0) => {
                    self.output.start_line();
                    self.output.append("• ");
                }
                Some(i) => {
                    self.output.start_line();
                    self.output.append(i.to_string().as_str());
                    self.output.append(". ");
                }
                None => (),
            },
            Action::Img {
                fname,
                url,
                xch_mode,
                is_map: _,
            }
            | Action::Image {
                fname,
                url,
                xch_mode,
                is_map: _,
            } => {
                if let Some(xch_mode) = xch_mode {
                    self.pueblo_active = true;
                    match xch_mode {
                        mxp::XchMode::Text => (),
                        mxp::XchMode::Html => self.suppress_newline = false,
                        mxp::XchMode::PureHtml => self.suppress_newline = true,
                    }
                }
                if let Some(url) = url {
                    let fname = fname.as_deref().unwrap_or_default();
                    self.output.append_image(format!("{url}{fname}"));
                }
            }
            Action::XchPage => {
                self.pueblo_active = true;
                self.mxp_off(false);
            }
            Action::Var { keywords, variable } => {
                if let Some(variable) = variable {
                    self.output
                        .set_mxp_variable(variable.into_owned(), keywords);
                }
            }
            Action::Sound
            | Action::Relocate
            | Action::Frame
            | Action::Dest
            | Action::Filter
            | Action::NoBr
            | Action::Strike
            | Action::Small
            | Action::Tt
            | Action::Samp
            | Action::Center
            | Action::Gauge
            | Action::Stat
            | Action::Expire { .. }
            | Action::SetOption
            | Action::RecommendOption
            | Action::Body
            | Action::Head
            | Action::Html
            | Action::Title
            | Action::XchPane => (),
        }
    }

    fn mxp_close_tags_from(&mut self, pos: usize) {
        if let Some(span_index) = self.mxp_active_tags.truncate(pos) {
            self.output.truncate_spans(span_index);
        }
    }

    fn mxp_collected_entity(&mut self) -> mxp::Result<()> {
        let mxp_string = self.take_mxp_string()?;
        let name = mxp_string.trim();
        mxp::validate(name, mxp::ErrorKind::InvalidEntityName)?;
        if let Some(entity) = self.mxp_state.get_entity(name)? {
            self.mxp_active = false;
            self.output.append(entity);
            self.mxp_active = true;
        }
        Ok(())
    }

    fn mxp_mode_change(&mut self, newmode: Option<mxp::Mode>) {
        let oldmode = self.mxp_mode;
        let newmode = newmode.unwrap_or(self.mxp_mode_default);
        let closing = oldmode.is_open() && !newmode.is_open();
        if closing {
            let closed = self.mxp_active_tags.last_unsecure_index();
            self.mxp_close_tags_from(closed);
        }
        match newmode {
            mxp::Mode::OPEN | mxp::Mode::SECURE | mxp::Mode::LOCKED => {
                self.mxp_mode_default = mxp::Mode::OPEN;
            }
            mxp::Mode::SECURE_ONCE => self.mxp_mode_previous = self.mxp_mode,
            mxp::Mode::PERM_OPEN | mxp::Mode::PERM_SECURE | mxp::Mode::PERM_LOCKED => {
                self.mxp_mode_default = newmode;
            }
            _ => (),
        }
        self.mxp_mode = newmode;
        if !newmode.is_user_defined() {
            return;
        }
        let mxp_state = mem::take(&mut self.mxp_state);
        if let Some(element) = mxp_state.get_line_tag(newmode) {
            if let Err(e) = self.mxp_open_element(element, &mxp::Arguments::new(), &mxp_state) {
                self.handle_mxp_error(e);
            }
        }
        self.mxp_state = mxp_state;
    }

    fn interpret_ansi(&mut self, code: u8) {
        match code {
            ansi::RESET => self.output.reset_ansi(),

            ansi::BOLD => self.output.set_ansi_flag(TextStyle::Bold),
            ansi::BLINK | ansi::SLOW_BLINK | ansi::FAST_BLINK => {
                self.output.set_ansi_flag(TextStyle::Italic);
            }
            ansi::UNDERLINE => self.output.set_ansi_flag(TextStyle::Underline),
            ansi::INVERSE => self.output.set_ansi_flag(TextStyle::Inverse),
            ansi::STRIKEOUT => self.output.set_ansi_flag(TextStyle::Strikeout),

            ansi::CANCEL_BOLD => self.output.unset_ansi_flag(TextStyle::Bold),
            ansi::CANCEL_BLINK | ansi::CANCEL_SLOW_BLINK | ansi::CANCEL_FAST_BLINK => {
                self.output.unset_ansi_flag(TextStyle::Italic);
            }
            ansi::CANCEL_UNDERLINE => self.output.unset_ansi_flag(TextStyle::Underline),
            ansi::CANCEL_INVERSE => self.output.unset_ansi_flag(TextStyle::Inverse),
            ansi::CANCEL_STRIKEOUT => self.output.unset_ansi_flag(TextStyle::Strikeout),

            ansi::FG_256_COLOR => self.phase = Phase::Foreground256Start,
            ansi::BG_256_COLOR => self.phase = Phase::Background256Start,
            ansi::FG_DEFAULT => self.output.set_ansi_foreground(TermColor::WHITE),
            ansi::BG_DEFAULT => self.output.set_ansi_background(TermColor::BLACK),
            ansi::FG_BLACK..=ansi::FG_WHITE => self
                .output
                .set_ansi_foreground(TermColor::Ansi(code - ansi::FG_BLACK)),
            ansi::BG_BLACK..=ansi::BG_WHITE => self
                .output
                .set_ansi_background(TermColor::Ansi(code - ansi::BG_BLACK)),
            _ => (),
        }
    }

    fn build_ansi_color(&self) -> RgbColor {
        RgbColor {
            r: self.ansi_red,
            g: self.ansi_green,
            b: self.ansi_blue,
        }
    }

    // See: https://en.wikipedia.org/wiki/ANSI_escape_code#8-bit
    fn interpret_256_ansi(&mut self, code: u8) {
        match self.phase {
            Phase::Foreground256Start => match code {
                5 => {
                    self.ansi_code = 0;
                    self.phase = Phase::Foreground256Finish;
                }
                2 => {
                    self.ansi_code = 0;
                    self.phase = Phase::Foreground24bFinish;
                }
                _ => self.phase = Phase::Normal,
            },
            Phase::Background256Start => match code {
                5 => {
                    self.ansi_code = 0;
                    self.phase = Phase::Background256Finish;
                }
                2 => {
                    self.ansi_code = 0;
                    self.phase = Phase::Background24bFinish;
                }
                _ => self.phase = Phase::Normal,
            },
            Phase::Foreground256Finish => {
                self.output
                    .set_ansi_foreground(RgbColor::xterm(self.ansi_code));
                self.phase = Phase::Normal;
            }
            Phase::Background256Finish => {
                self.output
                    .set_ansi_background(RgbColor::xterm(self.ansi_code));
                self.phase = Phase::Normal;
            }
            Phase::Foreground24bFinish => {
                self.ansi_red = code;
                self.phase = Phase::Foreground24brFinish;
            }
            Phase::Background24bFinish => {
                self.ansi_red = code;
                self.phase = Phase::Background24brFinish;
            }
            Phase::Foreground24brFinish => {
                self.ansi_green = code;
                self.phase = Phase::Foreground24bgFinish;
            }
            Phase::Background24brFinish => {
                self.ansi_green = code;
                self.phase = Phase::Background24bgFinish;
            }
            Phase::Foreground24bgFinish => {
                self.ansi_blue = code;
                self.phase = Phase::Foreground24bbFinish;
            }
            Phase::Background24bgFinish => {
                self.ansi_blue = code;
                self.phase = Phase::Background24bbFinish;
            }
            Phase::Foreground24bbFinish => {
                self.output.set_ansi_foreground(self.build_ansi_color());
                self.phase = Phase::Normal;
            }
            Phase::Background24bbFinish => {
                self.output.set_ansi_background(self.build_ansi_color());
                self.phase = Phase::Normal;
            }
            _ => (),
        }
    }

    fn interpret_code(&mut self) {
        if self.phase == Phase::DoingCode {
            self.interpret_ansi(self.ansi_code);
        } else {
            self.interpret_256_ansi(self.ansi_code);
        }
    }

    #[inline]
    pub fn receive(&mut self, bytes: &[u8], buf: &mut [u8]) -> io::Result<()> {
        if bytes.is_empty() {
            return Ok(());
        }
        let mut cursor = ReceiveCursor::new(bytes);
        if !self.decompressing {
            for byte in &mut cursor {
                self.receive_byte(byte);
                if self.decompressing {
                    break;
                }
            }
        }
        while !cursor.is_empty() {
            let n = self.decompress.decompress(&mut cursor, buf)?;
            let mut iter = buf[..n].iter();
            for &byte in &mut iter {
                self.receive_byte(byte);
                if !self.decompressing {
                    self.decompress.reset();
                    let remainder = iter.as_slice().to_vec();
                    self.receive(&remainder, buf)?;
                    return self.receive(bytes, buf);
                }
            }
        }

        Ok(())
    }

    #[allow(clippy::match_same_arms)]
    fn receive_byte(&mut self, c: u8) {
        let last_char = self.output.last().unwrap_or(b'\n');

        if last_char == b'\r' && c != b'\n' {
            self.output.append_effect(EffectFragment::CarriageReturn);
            return;
        }

        if self.phase == Phase::Utf8Character && !is_utf8_continuation(c) {
            self.output.append_utf8_char(&self.utf8_sequence);
            self.phase = Phase::Normal;
        }

        if self.phase.is_phase_reset(c) {
            if self.phase.is_mxp_mode_change() {
                self.mxp_mode_change(None);
            }
            self.phase = Phase::Normal;
        }

        match self.phase {
            Phase::Esc if c == b'[' => {
                self.phase = Phase::DoingCode;
                self.ansi_code = 0;
            }
            Phase::Esc => self.phase = Phase::Normal,

            Phase::Utf8Character => self.utf8_sequence.push(c),

            Phase::DoingCode
            | Phase::Foreground256Start
            | Phase::Foreground256Finish
            | Phase::Background256Start
            | Phase::Background256Finish
            | Phase::Foreground24bFinish
            | Phase::Foreground24brFinish
            | Phase::Foreground24bgFinish
            | Phase::Foreground24bbFinish
            | Phase::Background24bFinish
            | Phase::Background24brFinish
            | Phase::Background24bgFinish
            | Phase::Background24bbFinish => match c {
                b'm' => {
                    self.interpret_code();
                    self.phase = Phase::Normal;
                }
                b';' | b':' => {
                    self.interpret_code();
                    self.ansi_code = 0;
                }
                b'z' => {
                    let mode = mxp::Mode(self.ansi_code);
                    if mode == mxp::Mode::RESET {
                        self.mxp_off(false);
                    } else {
                        self.mxp_mode_change(Some(mode));
                    }
                    self.phase = Phase::Normal;
                }
                b'0'..=b'9' => {
                    self.ansi_code = ansi::append_digit_to_code(self.ansi_code, c);
                }
                _ => self.phase = Phase::Normal,
            },

            Phase::Iac if c == telnet::IAC => (),

            Phase::Iac => {
                self.subnegotiation_type = 0;
                match c {
                    telnet::EOR | telnet::GA => {
                        self.phase = Phase::Normal;
                        self.output.append_iac_ga();
                        if self.config.convert_ga_to_newline {
                            self.output.start_line();
                        }
                    }
                    telnet::SB => self.phase = Phase::Sb,
                    telnet::WILL => self.phase = Phase::Will,
                    telnet::WONT => self.phase = Phase::Wont,
                    telnet::DO => self.phase = Phase::Do,
                    telnet::DONT => self.phase = Phase::Dont,
                    telnet::AO => {
                        self.phase = Phase::Normal;
                        self.output.flush();
                    }
                    telnet::EC => {
                        self.phase = Phase::Normal;
                        self.output.append_effect(EffectFragment::EraseCharacter);
                    }
                    telnet::EL => {
                        self.phase = Phase::Normal;
                        self.output.append_effect(EffectFragment::EraseLine);
                    }
                    _ => self.phase = Phase::Normal,
                }
            }

            Phase::Will => {
                self.phase = Phase::Normal;
                let will = match c {
                    telnet::COMPRESS | telnet::COMPRESS2 if self.config.disable_compression => {
                        false
                    }
                    telnet::COMPRESS => !self.supports_mccp_2,
                    telnet::COMPRESS2 => {
                        self.supports_mccp_2 = true;
                        true
                    }
                    telnet::SGA | telnet::MUD_SPECIFIC | telnet::CHARSET => true,
                    telnet::ECHO if self.config.no_echo_off => false,
                    telnet::ECHO => {
                        self.no_echo = true;
                        true
                    }
                    telnet::MXP => match self.config.use_mxp {
                        UseMxp::Never => false,
                        UseMxp::Always | UseMxp::Command => true,
                        UseMxp::Query => {
                            self.mxp_on(false, false);
                            true
                        }
                    },
                    telnet::WILL_EOR => self.config.convert_ga_to_newline,
                    _ if self.config.will.contains(&c) => {
                        self.output.append_telnet_will(c);
                        true
                    }
                    _ => false,
                };
                self.input.append(&telnet::supports_do(c, will));
            }

            Phase::Wont => {
                self.phase = Phase::Normal;
                if !self.config.no_echo_off {
                    self.no_echo = false;
                }
                self.input.append(&telnet::supports_do(c, false));
            }

            Phase::Do => {
                self.phase = Phase::Normal;

                let will = match c {
                    telnet::SGA | telnet::MUD_SPECIFIC | telnet::ECHO | telnet::CHARSET => true,
                    telnet::TERMINAL_TYPE => {
                        self.ttype_sequence = 0;
                        true
                    }
                    telnet::NAWS if !self.config.naws => false,
                    telnet::NAWS => {
                        self.output.append_telnet_naws();
                        true
                    }
                    telnet::MXP => match self.config.use_mxp {
                        UseMxp::Never => false,
                        UseMxp::Always | UseMxp::Command => true,
                        UseMxp::Query => {
                            self.mxp_on(false, false);
                            true
                        }
                    },
                    _ if self.config.will.contains(&c) => {
                        self.output.append_telnet_do(c);
                        true
                    }
                    _ => false,
                };
                self.input.append(&telnet::supports_will(c, will));
            }

            Phase::Dont => {
                self.phase = Phase::Normal;
                match c {
                    telnet::MXP if self.mxp_active => self.mxp_off(true),
                    telnet::TERMINAL_TYPE => self.ttype_sequence = 0,
                    _ => (),
                }
                self.input.append(&telnet::supports_will(c, false));
            }

            Phase::Sb if c == telnet::COMPRESS => self.phase = Phase::Compress,
            Phase::Sb => {
                self.subnegotiation_type = c;
                self.subnegotiation_data.clear();
                self.phase = Phase::Subnegotiation;
            }

            Phase::Subnegotiation if c == telnet::IAC => self.phase = Phase::SubnegotiationIac,
            Phase::Subnegotiation => self.subnegotiation_data.push(c),

            Phase::Compress if c == telnet::WILL => self.phase = Phase::CompressWill,
            Phase::Compress => self.phase = Phase::Normal,

            Phase::CompressWill if c == telnet::SE => self.decompressing = true,
            Phase::CompressWill => self.phase = Phase::Normal,

            Phase::SubnegotiationIac if c == telnet::IAC => {
                self.subnegotiation_data.push(c);
                self.phase = Phase::Subnegotiation;
            }
            Phase::SubnegotiationIac => {
                self.phase = Phase::Normal;
                match self.subnegotiation_type {
                    telnet::COMPRESS2 => {
                        if !self.config.disable_compression {
                            self.decompressing = true;
                        }
                    }
                    telnet::MXP => {
                        if self.config.use_mxp == UseMxp::Command {
                            self.mxp_on(false, false);
                        }
                    }
                    telnet::TERMINAL_TYPE if !self.config.terminal_identification.is_empty() => {
                        if self.subnegotiation_data.first() == Some(&telnet::TTYPE_SEND) {
                            self.input.append(telnet::TTYPE_PREFIX);
                            let ttype = match self.ttype_sequence {
                                0 => {
                                    self.ttype_sequence += 1;
                                    let ttype = self.config.terminal_identification.as_bytes();
                                    if ttype.len() > 20 {
                                        &ttype[..20]
                                    } else {
                                        ttype
                                    }
                                }
                                1 => {
                                    self.ttype_sequence += 1;
                                    b"ANSI"
                                }
                                _ if self.config.disable_utf8 => b"MTTS 9",
                                _ => b"MTTS 13",
                            };
                            self.input.append(ttype);
                            self.input.append(telnet::TTYPE_SUFFIX);
                        }
                    }
                    telnet::CHARSET => {
                        let data = &self.subnegotiation_data;
                        if data.len() >= 3 && data[0] == 1 {
                            let charset = telnet::find_charset(data, !self.config.disable_utf8);
                            self.input.append(charset);
                        }
                    }
                    code => {
                        self.output
                            .append_subnegotiation(code, &self.subnegotiation_data);
                        self.subnegotiation_data.clear();
                    }
                }
            }

            Phase::MxpElement => match c {
                b'>' => {
                    if let Err(e) = self.mxp_collected_element() {
                        self.handle_mxp_error(e);
                    }
                    self.phase = Phase::Normal;
                }
                b'<' => {
                    self.mxp_string.push(c);
                    self.handle_mxp_error(mxp::Error::new(
                        &self.mxp_string,
                        mxp::ErrorKind::UnterminatedElement,
                    ));
                    self.mxp_string.clear();
                }
                b'\'' => {
                    const NON_ZERO_APOSTROPHE: NonZeroU8 = match NonZeroU8::new(b'\'') {
                        Some(quote) => quote,
                        None => unreachable!(),
                    };
                    self.mxp_string.push(c);
                    self.mxp_quote_terminator = Some(NON_ZERO_APOSTROPHE);
                    self.phase = Phase::MxpQuote;
                }
                b'"' => {
                    const NON_ZERO_QUOTE: NonZeroU8 = match NonZeroU8::new(b'"') {
                        Some(quote) => quote,
                        None => unreachable!(),
                    };
                    self.mxp_string.push(c);
                    self.mxp_quote_terminator = Some(NON_ZERO_QUOTE);
                    self.phase = Phase::MxpQuote;
                }
                b'-' => {
                    self.mxp_string.push(c);
                    if self.mxp_string.starts_with(b"!--") {
                        self.phase = Phase::MxpComment;
                    }
                }
                _ => self.mxp_string.push(c),
            },

            Phase::MxpComment if c == b'>' && self.mxp_string.ends_with(b"--") => {
                self.phase = Phase::Normal;
            }

            Phase::MxpComment => self.mxp_string.push(c),

            Phase::MxpQuote => {
                if let Some(terminator) = self.mxp_quote_terminator {
                    if terminator.get() == c {
                        self.phase = Phase::MxpElement;
                        self.mxp_quote_terminator = None;
                    }
                }
                self.mxp_string.push(c);
            }

            Phase::MxpEntity => match c {
                b';' => {
                    self.phase = Phase::Normal;
                    if let Err(e) = self.mxp_collected_entity() {
                        self.handle_mxp_error(e);
                    }
                }
                b'&' => {
                    self.mxp_string.push(c);
                    self.handle_mxp_error(mxp::Error::new(
                        &self.mxp_string,
                        mxp::ErrorKind::UnterminatedEntity,
                    ));
                    self.mxp_string.clear();
                }
                b'<' => {
                    self.mxp_string.push(c);
                    self.handle_mxp_error(mxp::Error::new(
                        &self.mxp_string,
                        mxp::ErrorKind::UnterminatedEntity,
                    ));
                    self.mxp_string.clear();
                    self.phase = Phase::MxpElement;
                }
                _ => self.mxp_string.push(c),
            },

            Phase::MxpWelcome => (),

            Phase::Normal => match c {
                telnet::ESC => self.phase = Phase::Esc,
                telnet::IAC => self.phase = Phase::Iac,
                // BEL
                0x07 => self.output.append_effect(EffectFragment::Beep),
                // BS
                0x08 => self.output.append_effect(EffectFragment::Backspace),
                // FF
                0x0C => self.output.append_page_break(),
                b'\t' if self.output.format().contains(TextFormat::Paragraph) => {
                    if last_char != b' ' {
                        self.output.append(" ");
                    }
                }
                b'\r' => (),
                b' ' if last_char == b' '
                    && self.output.format().contains(TextFormat::Paragraph) => {}
                b'\n' => {
                    if self.mxp_active && !self.pueblo_active {
                        self.mxp_mode_change(None);
                    }
                    let format = self.output.format();
                    if format.contains(TextFormat::Paragraph) {
                        match last_char {
                            b'\n' => {
                                self.output.start_line();
                                self.output.start_line();
                            }
                            b'.' => self.output.append("  "),
                            b' ' | b'\t' | 0x0C => (),
                            _ => self.output.append(" "),
                        }
                    } else if !self.suppress_newline && !format.contains(TextFormat::Pre) {
                        self.output.start_line();
                    }
                }
                _ if is_utf8_higher_order(c) => {
                    self.utf8_sequence.push(c);
                    self.phase = Phase::Utf8Character;
                }
                _ if !self.mxp_active || !self.mxp_mode.is_mxp() => self.output.push(c),
                b'<' => {
                    self.mxp_string.clear();
                    self.phase = Phase::MxpElement;
                }
                _ if self.mxp_mode == mxp::Mode::SECURE_ONCE => {
                    self.mxp_mode = self.mxp_mode_previous;
                }
                _ if self.mxp_script => (),
                b'&' => {
                    self.mxp_string.clear();
                    self.phase = Phase::MxpEntity;
                }
                _ => self.output.push(c),
            },
        }
    }
}

pub const fn is_utf8_higher_order(c: u8) -> bool {
    (c & 0x80) != 0
}

pub const fn is_utf8_continuation(c: u8) -> bool {
    (c & 0xC0) != 0x80
}
