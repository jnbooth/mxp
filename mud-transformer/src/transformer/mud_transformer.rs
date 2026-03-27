use std::borrow::Cow;
use std::num::NonZero;
use std::{mem, slice};

use bytes::BytesMut;
use bytestring::ByteString;
use bytestringmut::ByteStringMut;
use log::{error, info, warn};
use mxp::element::ElementFlag;
use mxp::entity::PublishedIter;
use mxp::node::{Definition, Tag, TagOpen};

use super::byteset::ByteSet;
use super::config::{TabBehavior, TransformerConfig, UseMxp};
use super::phase::Phase;
use super::state::StateLock;
use super::tag_list::TagList;
use crate::bytestring_ext::ByteStringMutExt;
use crate::escape::{ansi, telnet};
use crate::input::{BufferedInput, InputDrain};
use crate::opt::{self, charset, mccp, mnes, mtts, status};
use crate::output::{
    BufferedOutput, ControlFragment, EntityFragment, MapperFragment, MxpFragment, OutputDrain,
    OutputFragment, TelnetFragment, TextStyle, VariableFragment,
};
use crate::protocol::{Negotiate, TelnetSource, TelnetVerb, xterm};
use crate::term::{CursorEffect, EraseRange, EraseTarget};

fn input_mxp_auth(input: &mut BufferedInput, auth: &str) {
    if auth.is_empty() {
        return;
    }
    write!(input, "{auth}\r\n");
}

#[derive(Debug)]
pub struct Transformer {
    config: TransformerConfig,

    phase: Phase,
    doing: Box<ByteSet>,
    in_paragraph: bool,
    ignore_next_newline: bool,

    mxp_active: bool,
    mxp_buf: ByteStringMut,
    mxp_entity_string: Vec<u8>,
    mxp_mode: mxp::ModeState,
    mxp_quote_terminator: Option<NonZero<u8>>,
    mxp_state: StateLock,
    mxp_tags: TagList,

    charsets: charset::Charsets,
    decompress: mccp::Decompress,
    decompressing: bool,
    mnes_variables: mnes::Variables,
    ttype_negotiator: mtts::Negotiator,

    ansi: xterm::Interpreter,
    after_ansi: bool,
    subnegotiation_data: BytesMut,
    subnegotiation_type: u8,
    last_char: u8,
    utf8_sequence: Vec<u8>,

    input: BufferedInput,
    output: BufferedOutput,
}

impl Default for Transformer {
    fn default() -> Self {
        Self::new(TransformerConfig::default())
    }
}

impl Transformer {
    pub fn new(mut config: TransformerConfig) -> Self {
        let mut output = BufferedOutput::new();
        output.set_colors(&config.colors);
        if config.ignore_mxp_colors {
            output.disable_mxp_colors();
        }
        config.postprocess_will();
        Self {
            phase: Phase::Normal,
            doing: Box::default(),

            mxp_active: config.use_mxp == UseMxp::Always,

            in_paragraph: false,
            ignore_next_newline: false,
            mxp_buf: ByteStringMut::new(),
            mxp_mode: mxp::ModeState::new(),
            mxp_quote_terminator: None,
            mxp_entity_string: Vec::new(),
            mxp_tags: TagList::new(),
            mxp_state: mxp::State::with_globals().into(),

            charsets: charset::Charsets::new(),
            decompress: mccp::Decompress::new(),
            decompressing: false,
            mnes_variables: mnes::Variables::new(),
            ttype_negotiator: mtts::Negotiator::new(),

            ansi: xterm::Interpreter::new(),
            after_ansi: false,
            subnegotiation_type: 0,
            subnegotiation_data: BytesMut::new(),

            last_char: b'\n',
            utf8_sequence: Vec::with_capacity(4),
            output,
            input: BufferedInput::new(),

            config,
        }
    }

    pub fn decompressing(&self) -> bool {
        self.decompressing
    }

    pub fn mxp_active(&self) -> bool {
        self.mxp_active
    }

    pub fn get_mxp_entity(&self, name: &str) -> Option<&str> {
        self.mxp_state.get_entity(name)
    }

    pub fn set_mxp_entity(&mut self, name: String, value: String) -> bool {
        self.mxp_state.entities_mut().insert(name, value).is_ok()
    }

    pub fn unset_mxp_entity(&mut self, name: &str) -> bool {
        self.mxp_state
            .entities_mut()
            .remove(name)
            .is_ok_and(|entity| entity.is_some())
    }

    pub fn config(&self) -> &TransformerConfig {
        &self.config
    }

    pub fn set_config(&mut self, mut config: TransformerConfig) {
        mem::swap(&mut self.config, &mut config);
        self.config.postprocess_will();
        if self.config.ignore_mxp_colors {
            self.output.disable_mxp_colors();
        } else {
            self.output.enable_mxp_colors();
        }
        if config.colors != self.config.colors {
            self.output.set_colors(&self.config.colors);
        }
        match self.config.use_mxp {
            UseMxp::Always => self.mxp_on(),
            UseMxp::Never => self.mxp_off(),
            UseMxp::Command | UseMxp::Query => (),
        }
        let mnes_updates = self.mnes_variables.changes(&config, &self.config);
        if mnes_updates.is_empty() {
            return;
        }
        self.send_subnegotiation(mnes_updates);
    }

    pub fn xterm_color(&self, i: u8) -> mxp::RgbColor {
        self.output.get_xterm_color(i)
    }

    pub fn set_xterm_color(&mut self, i: u8, color: mxp::RgbColor) {
        self.output.set_xterm_color(i, color);
    }

    pub fn xterm_colors(&self) -> &[mxp::RgbColor; 256] {
        self.output.xterm_colors()
    }

    pub fn xterm_colors_mut(&mut self) -> &mut [mxp::RgbColor; 256] {
        self.output.xterm_colors_mut()
    }

    pub fn has_output(&self) -> bool {
        !self.output.is_empty()
    }

    pub fn drain_output(&mut self) -> OutputDrain<'_> {
        self.output.drain_complete()
    }

    pub fn flush_output(&mut self) -> OutputDrain<'_> {
        self.output.flush();
        self.output.drain()
    }

    pub fn drain_input(&mut self) -> Option<InputDrain<'_>> {
        self.input.drain()
    }

    pub fn published_entities(&self) -> PublishedIter<'_> {
        self.mxp_state.published_entities()
    }

    pub fn mxp_element_count(&self) -> usize {
        self.mxp_state.custom_elements_len()
    }

    pub fn mxp_entity_count(&self) -> usize {
        self.mxp_state.custom_entities_len()
    }

    pub fn reset_ansi(&mut self) {
        self.output.reset_ansi();
    }

    pub fn reset_mxp(&mut self) {
        self.output.reset_mxp();
    }

    pub fn receive(&mut self, mut bytes: &[u8], buf: &mut [u8]) -> usize {
        let initial_len = bytes.len();
        if !self.decompressing {
            bytes = self.receive_bytes(bytes);
        }
        if bytes.is_empty() {
            return bytes.len();
        }
        let mut received = initial_len - bytes.len();
        while self.decompressing {
            let (n, status) = self.decompress.decompress(&mut bytes, buf);
            let finished = status != Ok(mccp::Status::Ok);
            if finished {
                info!(target: "mud", "[DECOMPRESS] Ending gracefully");
            }
            if n == 0 {
                self.decompressing = !finished;
                break;
            }
            received += n;
            self.receive_bytes(&buf[..n]);
            if self.decompressing
                && let Err(e) = status
            {
                error!(target: "mud", "[DECOMPRESS] {e}");
                self.send_negotiation(TelnetVerb::Dont, opt::MCCP2);
            }
            if finished {
                self.decompressing = false;
            }
        }
        if bytes.is_empty() {
            return received;
        }
        received + self.receive(bytes, buf)
    }

    fn receive_bytes<'a>(&mut self, bytes: &'a [u8]) -> &'a [u8] {
        if bytes.is_empty() {
            return bytes;
        }
        let decompressing = self.decompressing;
        let mut iter = bytes.iter();
        for &byte in &mut iter {
            self.receive_byte(byte);
            if self.decompressing != decompressing {
                break;
            }
        }
        iter.as_slice()
    }

    fn send_negotiation(&mut self, verb: TelnetVerb, code: u8) {
        info!(target: "mud", "[TELNET] Sending IAC {verb:?} {code}");
        self.input.write(&[telnet::IAC, verb as u8, code]);
        self.output.append(TelnetFragment::Negotiation {
            source: TelnetSource::Client,
            verb,
            code,
        });
    }

    fn send_subnegotiation<T: Negotiate>(&mut self, negotiator: T) {
        self.input.write(&[telnet::IAC, telnet::SB, T::OPT]);
        negotiator.negotiate(&mut self.input, &self.config).unwrap();
        self.input.write(&[telnet::IAC, telnet::SE]);
    }

    fn receive_subnegotiation(&mut self, subnegotiation_type: u8, data: &[u8]) {
        match subnegotiation_type {
            opt::STATUS => {
                if data != [status::SEND] {
                    return;
                }
                self.input
                    .write(&[telnet::IAC, telnet::SB, opt::STATUS, status::IS]);
                status::encode(&mut self.input, telnet::WILL, &self.config.will).unwrap();
                status::encode(&mut self.input, telnet::DO, &*self.doing).unwrap();
                self.input.write(&[telnet::IAC, telnet::SE]);
            }
            opt::MTTS => {
                if data != [mtts::SEND] || self.config.terminal_identification.is_empty() {
                    return;
                }
                self.send_subnegotiation(self.ttype_negotiator);
                self.ttype_negotiator.advance();
            }
            opt::MNES => {
                let [mnes::SEND, request @ ..] = data else {
                    return;
                };
                self.mnes_variables = mnes::Variables::from(request);
                self.send_subnegotiation(self.mnes_variables);
            }
            opt::CHARSET => {
                let [charset::REQUEST, request @ ..] = data else {
                    return;
                };
                self.charsets = match charset::Charsets::decode(request) {
                    Ok(charsets) => charsets,
                    Err(e) => {
                        error!(target: "mud", "[TELNET] Error decoding charsets: {e}");
                        return;
                    }
                };
                self.send_subnegotiation(self.charsets);
            }
            opt::MCCP2 => {
                if !self.config.disable_compression {
                    info!(target: "mud", "[DECOMPRESS] Beginning decompression");
                    self.decompressing = true;
                }
            }
            opt::MXP => {
                if self.config.use_mxp == UseMxp::Command {
                    self.mxp_on();
                }
            }
            _ => (),
        }
    }

    fn mxp_close_tags_from(&mut self, pos: usize) {
        if let Some(span_index) = self.mxp_tags.truncate(pos) {
            self.output.truncate_spans(span_index, &mut self.mxp_state);
        }
    }

    fn mxp_on(&mut self) {
        if self.mxp_active {
            return;
        }
        info!(target: "mxp", "MXP enabled");
        self.mxp_active = true;
        self.output.append(TelnetFragment::Mxp { enabled: true });
        self.mxp_mode.set(mxp::Mode::RESET);
        self.mxp_tags.clear();
        self.mxp_state.clear();
    }

    fn mxp_reset(&mut self) {
        self.output.reset_mxp();
        self.mxp_close_tags_from(0);
    }

    fn mxp_off(&mut self) {
        if !self.mxp_active {
            return;
        }
        info!(target: "mxp", "MXP disabled");
        self.mxp_reset();
        self.mxp_mode_change(Some(mxp::Mode::RESET));
        if self.phase.is_mxp() {
            self.phase = Phase::Normal;
        }
        self.mxp_active = false;
        self.output.append(TelnetFragment::Mxp { enabled: false });
    }

    fn mxp_mode_change(&mut self, newmode: Option<mxp::Mode>) {
        if newmode == Some(mxp::Mode::RESET) {
            self.mxp_reset();
            return;
        }
        let should_close = if let Some(newmode) = newmode {
            self.mxp_mode.set(newmode)
        } else {
            self.mxp_mode.revert();
            true
        };
        if should_close {
            let close_from = self.mxp_tags.last_open_index();
            self.mxp_close_tags_from(close_from);
        }
        if let Some(parse_as) = newmode.and_then(mxp::Mode::parse_as) {
            self.output.set_mxp_parse_as(parse_as);
        }
        if !self.mxp_mode.is_user_defined() {
            return;
        }
        let mxp_state = self.mxp_state.take();
        if let Err(e) = self.mxp_set_line_tag(&mxp_state) {
            warn!(target: "mxp", "{e}");
        }
        self.mxp_state.set(mxp_state);
    }

    fn mxp_set_line_tag(&mut self, mxp_state: &mxp::State) -> mxp::Result<()> {
        let Some(line_tag) = self.mxp_mode.line_tag(mxp_state) else {
            return Ok(());
        };
        self.output.set_mxp_line_tag(line_tag.properties);
        let Some(element) = line_tag.element else {
            return Ok(());
        };
        self.mxp_open_element(element, &mxp::Arguments::new(), false, mxp_state)
    }

    fn mxp_collect_entity(&mut self) -> mxp::Result<()> {
        let name = mxp::validate_utf8(&self.mxp_entity_string)?;
        let entity = self.mxp_state.decode_entity(name)?;
        write!(self.output, "{entity}");
        Ok(())
    }

    fn mxp_collect_element(&mut self, entity_string: &[u8]) -> mxp::Result<()> {
        let secure = self.mxp_mode.use_secure();
        let source = mxp::validate_utf8(entity_string)?;
        match Tag::parse(source, secure)? {
            Tag::Definition(definition) => self.mxp_define(definition),
            Tag::Close(tag) => {
                let closed = self.mxp_tags.find_last(secure, tag.name)?;
                self.mxp_close_tags_from(closed);
                Ok(())
            }
            Tag::Open(tag) => {
                let mxp_state = self.mxp_state.take();
                let result = self.mxp_start_tag(&tag, secure, &mxp_state);
                self.mxp_state.set(mxp_state);
                result
            }
        }
    }

    fn mxp_define(&mut self, definition: Definition) -> mxp::Result<()> {
        let name = definition.name();
        if let Some(entry) = self.mxp_state.define(definition)? {
            self.output.append(EntityFragment {
                name: self.mxp_buf.share(name),
                value: entry.value.map(|value| self.mxp_buf.share(value)),
                publish: entry.publish,
            });
        }
        Ok(())
    }

    fn mxp_start_tag(
        &mut self,
        tag: &TagOpen,
        secure: bool,
        mxp_state: &mxp::State,
    ) -> mxp::Result<()> {
        let component = mxp_state.get_component(tag.name, secure)?;

        if !component.is_command() {
            self.mxp_tags
                .open(component, secure, self.output.span_len());
        }

        match component {
            mxp::Component::AtomicTag(atom) => {
                let action = atom.decode(&tag.arguments, mxp_state)?;
                self.mxp_apply_action(action);
                Ok(())
            }
            mxp::Component::Element(el) => {
                if let Some(line_tag) = el.line_tag_properties(mxp_state) {
                    self.output.set_mxp_line_tag(line_tag);
                }
                self.mxp_open_element(el, &tag.arguments, tag.empty, mxp_state)
            }
        }
    }

    fn mxp_set_flag(&mut self, flag: &ElementFlag, args: &mxp::Arguments, empty: bool) {
        if !empty {
            match flag {
                ElementFlag::ParseAs(parse_as) => self.output.set_mxp_parse_as(*parse_as),
                ElementFlag::Set(variable) => self.output.set_mxp_variable(variable),
            }
            return;
        }
        let arg = match args.at(0) {
            Some(value) => self.mxp_buf.share(value),
            None => ByteString::new(),
        };
        match flag {
            ElementFlag::ParseAs(parse_as) => self.output.append(MapperFragment {
                parse_as: *parse_as,
                value: arg,
            }),
            ElementFlag::Set(variable) => self.output.append(VariableFragment {
                name: self.mxp_buf.share(variable),
                value: arg,
            }),
        }
    }

    fn mxp_open_element(
        &mut self,
        el: &mxp::Element,
        args: &mxp::Arguments,
        empty: bool,
        mxp_state: &mxp::State,
    ) -> mxp::Result<()> {
        if let Some(flag) = &el.flag {
            self.mxp_set_flag(flag, args, empty || el.empty);
        }
        for action in el.decode(args, mxp_state) {
            self.mxp_apply_action(action?);
        }
        Ok(())
    }

    fn mxp_apply_action(&mut self, action: mxp::Action<Cow<str>>) {
        use mxp::Action;

        match action {
            Action::Bold => self.output.set_mxp_style(TextStyle::Bold),
            Action::Br => self.output.start_line(),
            Action::Color(m) => self.output.set_mxp_color(m),
            Action::Dest(m) => self.output.set_mxp_window(m),
            Action::Expire(m) => self.output.append(m.into_owned()),
            Action::Filter(m) => self.output.append(m.into_owned()),
            Action::Font(m) => self.output.set_mxp_font(m),
            Action::Frame(m) => self.output.append(m.into_owned()),
            Action::Gauge(m) => self.output.append(m.into_owned()),
            Action::Heading(m) => self.output.set_mxp_heading(m),
            Action::Highlight => self.output.set_mxp_style(TextStyle::Highlight),
            Action::Hr => self.output.append(OutputFragment::Hr),
            Action::Hyperlink(m) => self.output.set_mxp_link(m.into_owned()),
            Action::Image(m) => self.output.append(m.into_owned()),
            Action::Italic => self.output.set_mxp_style(TextStyle::Italic),
            Action::Music(m) => self.output.append(m.into_owned()),
            Action::MusicOff => self.output.append(MxpFragment::MusicOff),
            Action::MxpOff | Action::Reset => (),
            Action::NoBr => self.ignore_next_newline = true,
            Action::P => self.in_paragraph = true,
            Action::Password => input_mxp_auth(&mut self.input, &self.config.password),
            Action::Relocate(m) => self.output.append(m.into_owned()),
            Action::SBr => self.output.write_str(" "),
            Action::Send(m) => self.output.set_mxp_link(m.into_owned()),
            Action::Small => self.output.set_mxp_style(TextStyle::Small),
            Action::Sound(m) => self.output.append(m.into_owned()),
            Action::SoundOff => self.output.append(MxpFragment::SoundOff),
            Action::Stat(m) => self.output.append(m.into_owned()),
            Action::Strikeout => self.output.set_mxp_style(TextStyle::Strikeout),
            Action::StyleVersion(m) => {
                let m = m.into_owned();
                self.config.style_version = Some(m.styleversion.clone());
                self.output.append(m);
            }
            Action::Support(m) => write!(self.input, "{}", self.config.support_response(m)),
            Action::Tt => self.output.set_mxp_style(TextStyle::NonProportional),
            Action::Underline => self.output.set_mxp_style(TextStyle::Underline),
            Action::User => input_mxp_auth(&mut self.input, &self.config.player),
            Action::Var(m) => self.output.set_mxp_entity(m),
            Action::Version => write!(self.input, "{}", self.config.version_response()),
        }
    }

    fn mxp_unterminated(&mut self, error: mxp::ErrorKind) {
        let entity_string = String::from_utf8_lossy(&self.mxp_entity_string);
        let e = mxp::Error::new(entity_string, error);
        warn!(target: "mxp", "{e}");
        self.mxp_entity_string.clear();
        self.mxp_mode.use_secure(); // clear SECURE_ONCE
    }

    #[allow(clippy::match_same_arms)]
    fn receive_byte(&mut self, c: u8) {
        if self.after_ansi && c != ansi::ESC && !self.phase.is_ansi() {
            self.after_ansi = false;
            self.ansi.clear_mslp_link();
        }

        if self.last_char == b'\r' && c != b'\n' {
            self.output.append(ControlFragment::CarriageReturn);
        }

        if self.phase == Phase::Utf8Character && !is_utf8_continuation(c) {
            let sequence = match str::from_utf8(&self.utf8_sequence) {
                Ok(sequence) => sequence,
                Err(e) => {
                    warn!(target: "mud", "[TELNET] Malformed UTF-8: {:?} ({e})", self.utf8_sequence);
                    "\u{FFFD}"
                }
            };
            self.output.write_str(sequence);
            self.phase = Phase::Normal;
        }

        if self.phase.is_phase_reset(c) {
            self.phase = Phase::Normal;
            match self.phase {
                Phase::Ansi | Phase::AnsiString | Phase::Esc => self.ansi.terminate(),
                Phase::MxpComment => self.mxp_unterminated(mxp::ErrorKind::UnterminatedComment),
                Phase::MxpElement => self.mxp_unterminated(mxp::ErrorKind::UnterminatedElement),
                Phase::MxpEntity => self.mxp_unterminated(mxp::ErrorKind::UnterminatedEntity),
                Phase::MxpQuote => self.mxp_unterminated(mxp::ErrorKind::UnterminatedQuote),
                _ => (),
            }
        }

        match self.phase {
            Phase::Normal => {
                let last_char = self.last_char;
                self.last_char = c;
                if self.mxp_active && c != b'<' && self.mxp_mode.is_secure_once() {
                    let e = mxp::Error::new(c as char, mxp::ErrorKind::TextAfterSecureOnce);
                    warn!(target: "mxp", "{e}");
                    self.mxp_mode.use_secure();
                }
                match c {
                    b' ' if self.in_paragraph && last_char == b' ' => (),
                    b'<' if self.mxp_active && !self.mxp_mode.is_locked() => {
                        self.mxp_entity_string.clear();
                        self.phase = Phase::MxpElement;
                    }
                    b'&' if self.mxp_active && !self.mxp_mode.is_locked() => {
                        self.mxp_entity_string.clear();
                        self.phase = Phase::MxpEntity;
                    }
                    32..=126 => {
                        // SAFETY: `utf8` is valid UTF-8, since it is a single ASCII byte.
                        let one_ascii = unsafe { str::from_utf8_unchecked(slice::from_ref(&c)) };
                        self.output.write_str(one_ascii);
                    }
                    b'\n' => {
                        if self.mxp_active {
                            self.mxp_mode_change(None);
                        }
                        if self.in_paragraph {
                            match last_char {
                                b'\n' => {
                                    self.output.start_line();
                                    self.output.start_line();
                                }
                                b'.' => self.output.write_str("  "),
                                b' ' | b'\t' | 0x0C => (),
                                _ => self.output.write_str(" "),
                            }
                        } else if self.ignore_next_newline {
                            self.ignore_next_newline = false;
                        } else {
                            self.output.start_line();
                        }
                    }
                    b'\r' => (),
                    b'\t' if self.in_paragraph => {
                        if last_char != b' ' {
                            self.output.write_str(" ");
                        }
                    }
                    b'\t' => match self.config.tab {
                        TabBehavior::Control => self.output.append(CursorEffect::TabForward(1)),
                        TabBehavior::NextMultipleOf8 => self.output.append_tab(),
                        tab => self.output.write_str(tab.str()),
                    },
                    ansi::ESC => self.phase = Phase::Esc,
                    telnet::IAC => self.phase = Phase::Iac,
                    ansi::ENQ => self.input.write(self.ansi.answerback()),
                    ansi::BEL => self.output.append(ControlFragment::Beep),
                    ansi::BS => self.output.append(CursorEffect::Back(1)),
                    ansi::VT => self.output.append(ControlFragment::VerticalTab),
                    ansi::FF => self.output.append(OutputFragment::PageBreak),
                    128..248 => {
                        self.utf8_sequence.push(c);
                        self.phase = Phase::Utf8Character;
                    }
                    ..32 | ansi::DEL | 248.. => {
                        info!(target: "mud", "[TELNET] Unhandled control character: {c}");
                    }
                }
            }

            Phase::Esc if c == ansi::ESC => (),

            Phase::Esc => {
                self.phase = match self.ansi.escape(c, &mut self.output, &mut self.input) {
                    xterm::Start::Continue => Phase::Ansi,
                    xterm::Start::BeginString => Phase::AnsiString,
                    xterm::Start::Done => {
                        self.after_ansi = true;
                        Phase::Normal
                    }
                }
            }

            Phase::Utf8Character => self.utf8_sequence.push(c),

            Phase::Ansi | Phase::AnsiString if c == ansi::ESC => {
                self.phase = Phase::Esc;
            }

            Phase::Ansi | Phase::AnsiString => {
                let mut reset = true;
                match self.ansi.interpret(c, &mut self.output, &mut self.input) {
                    xterm::Outcome::Continue => reset = false,
                    xterm::Outcome::Fail => {
                        let ansi = self.ansi.sequence();
                        warn!(target: "mud", "[TELNET] Escape sequence failed: {ansi}");
                    }
                    xterm::Outcome::Done => (),
                    xterm::Outcome::Link => {
                        if let Some(link) = self.ansi.take_mslp_link() {
                            self.output.set_mxp_link(link);
                        } else if self.config.linkify_underlined {
                            self.output.set_mxp_link(mxp::Send::default());
                        }
                    }
                    xterm::Outcome::Mxp(mode) => self.mxp_mode_change(Some(mode)),
                }
                if reset {
                    self.phase = Phase::Normal;
                    self.after_ansi = true;
                }
            }

            Phase::Iac if c == telnet::IAC => (),

            Phase::Iac => {
                self.subnegotiation_type = 0;
                match c {
                    telnet::EOR | telnet::GA => {
                        self.phase = Phase::Normal;
                        self.output.append(TelnetFragment::GoAhead);
                        if c == telnet::GA && self.config.convert_ga_to_newline {
                            self.output.start_line();
                        }
                    }
                    telnet::AO => {
                        self.phase = Phase::Normal;
                        self.output.flush();
                    }
                    telnet::AYT => self.input.write_str("YES"),
                    telnet::EC => {
                        self.phase = Phase::Normal;
                        self.output.append(CursorEffect::Back(1));
                    }
                    telnet::EL => {
                        self.phase = Phase::Normal;
                        self.output.append(ControlFragment::Erase {
                            target: EraseTarget::Line,
                            range: EraseRange::Full,
                            selective: false,
                        });
                    }
                    telnet::SB => self.phase = Phase::Sb,
                    telnet::WILL => self.phase = Phase::Will,
                    telnet::WONT => self.phase = Phase::Wont,
                    telnet::DO => self.phase = Phase::Do,
                    telnet::DONT => self.phase = Phase::Dont,
                    _ => self.phase = Phase::Normal,
                }
            }

            Phase::Will => {
                self.phase = Phase::Normal;
                self.output.append(TelnetFragment::Negotiation {
                    source: TelnetSource::Server,
                    verb: TelnetVerb::Will,
                    code: c,
                });
                let supported = self.config.will.contains(c);
                if supported {
                    match c {
                        opt::ECHO => self
                            .output
                            .append(TelnetFragment::SetEcho { should_echo: false }),
                        opt::MXP if self.config.use_mxp == UseMxp::Query => self.mxp_on(),
                        _ => (),
                    }
                }
                let verb = if supported {
                    TelnetVerb::Do
                } else {
                    TelnetVerb::Dont
                };
                self.send_negotiation(verb, c);
            }

            Phase::Wont => {
                self.phase = Phase::Normal;
                self.output.append(TelnetFragment::Negotiation {
                    source: TelnetSource::Server,
                    verb: TelnetVerb::Wont,
                    code: c,
                });
                if self.config.will.contains(c) {
                    match c {
                        opt::ECHO => self
                            .output
                            .append(TelnetFragment::SetEcho { should_echo: true }),
                        opt::MCCP2 => {
                            info!(target: "mud", "[DECOMPRESS] Decompression disabled");
                            self.decompressing = false;
                        }
                        _ => (),
                    }
                }
                self.send_negotiation(TelnetVerb::Dont, c);
            }

            Phase::Do => {
                self.phase = Phase::Normal;
                self.output.append(TelnetFragment::Negotiation {
                    source: TelnetSource::Server,
                    verb: TelnetVerb::Do,
                    code: c,
                });
                let supported = self.config.will.contains(c);
                if supported {
                    self.doing.insert(c);
                    match c {
                        opt::MTTS => self.ttype_negotiator.reset(),
                        opt::NAWS => self.output.append(TelnetFragment::Naws),
                        opt::MXP if self.config.use_mxp == UseMxp::Query => self.mxp_on(),
                        _ => (),
                    }
                }
                let verb = if supported {
                    TelnetVerb::Will
                } else {
                    TelnetVerb::Wont
                };
                self.send_negotiation(verb, c);
            }

            Phase::Dont => {
                self.doing.remove(c);
                self.output.append(TelnetFragment::Negotiation {
                    source: TelnetSource::Server,
                    verb: TelnetVerb::Dont,
                    code: c,
                });
                self.phase = Phase::Normal;
                match c {
                    opt::MXP if self.mxp_active => self.mxp_off(),
                    opt::MTTS => self.ttype_negotiator.reset(),
                    opt::MNES => self.mnes_variables.clear(),
                    _ => (),
                }
                self.send_negotiation(TelnetVerb::Wont, c);
            }

            Phase::Sb => {
                self.subnegotiation_type = c;
                self.subnegotiation_data.clear();
                self.phase = Phase::Subnegotiation;
            }

            Phase::Subnegotiation if c == telnet::IAC => self.phase = Phase::SubnegotiationIac,
            Phase::Subnegotiation => self.subnegotiation_data.extend_from_slice(&[c]),

            Phase::SubnegotiationIac if c == telnet::IAC => {
                self.subnegotiation_data.extend_from_slice(&[c]);
                self.phase = Phase::Subnegotiation;
            }
            Phase::SubnegotiationIac => {
                self.phase = Phase::Normal;
                let data = self.subnegotiation_data.split().freeze();
                if c != telnet::SE {
                    error!(target: "mud",
                        "[TELNET] subnegotiation terminated with {c} instead of SE: {data:?}"
                    );
                }
                self.receive_subnegotiation(self.subnegotiation_type, &data);
                self.output.append(TelnetFragment::Subnegotiation {
                    code: self.subnegotiation_type,
                    data,
                });
            }

            Phase::MxpElement => match c {
                b'>' => {
                    self.phase = Phase::Normal;
                    let mut entity_string = Vec::new(); // never allocates
                    mem::swap(&mut entity_string, &mut self.mxp_entity_string);
                    if let Err(mut e) = self.mxp_collect_element(&entity_string) {
                        if let Ok(source) = str::from_utf8(&entity_string) {
                            e = e.with_context(format_args!(" (in <{source}>)"));
                        }
                        warn!(target: "mxp", "{e}");
                    }
                    mem::swap(&mut entity_string, &mut self.mxp_entity_string);
                    self.mxp_entity_string.clear();
                }
                b'<' => {
                    self.mxp_unterminated(mxp::ErrorKind::UnterminatedElement);
                }
                b'\'' | b'"' => {
                    self.mxp_entity_string.push(c);
                    self.mxp_quote_terminator = NonZero::new(c);
                    self.phase = Phase::MxpQuote;
                }
                b'-' => {
                    self.mxp_entity_string.push(c);
                    if self.mxp_entity_string.starts_with(b"!--") {
                        self.phase = Phase::MxpComment;
                    }
                }
                _ => self.mxp_entity_string.push(c),
            },

            Phase::MxpComment => match c {
                b'>' if self.mxp_entity_string.ends_with(b"--") => self.phase = Phase::Normal,
                _ => self.mxp_entity_string.push(c),
            },

            Phase::MxpQuote => {
                self.mxp_entity_string.push(c);
                if let Some(terminator) = self.mxp_quote_terminator
                    && terminator.get() == c
                {
                    self.phase = Phase::MxpElement;
                    self.mxp_quote_terminator = None;
                }
            }

            Phase::MxpEntity => match c {
                b';' => {
                    self.phase = Phase::Normal;
                    if let Err(e) = self.mxp_collect_entity() {
                        warn!(target: "mxp", "{e}");
                    }
                }
                b'&' => self.mxp_unterminated(mxp::ErrorKind::UnterminatedEntity),
                b'<' => {
                    self.mxp_unterminated(mxp::ErrorKind::UnterminatedEntity);
                    self.phase = Phase::MxpElement;
                }
                _ => self.mxp_entity_string.push(c),
            },
        }
    }
}

#[inline]
const fn is_utf8_continuation(c: u8) -> bool {
    (c & 0xC0) == 0x80
}
