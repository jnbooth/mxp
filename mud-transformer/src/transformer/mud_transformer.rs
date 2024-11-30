use std::borrow::Cow;
use std::fmt::Write;
use std::num::NonZeroU8;
use std::{io, mem};

use super::config::{TransformerConfig, UseMxp};
use super::cursor::ReceiveCursor;
use super::input::{BufferedInput, Drain as InputDrain};
use super::phase::Phase;
use super::tag::{Tag, TagList};
use crate::output::{
    BufferedOutput, EffectFragment, EntityFragment, EntitySetter, OutputDrain, OutputFragment,
    TelnetFragment, TelnetSource, TelnetVerb, TextStyle,
};
use crate::protocol::{self, ansi, charset, mccp, mnes, mssp, mtts, Negotiate};
use enumeration::EnumSet;
use mxp::escape::telnet;

fn input_mxp_auth(input: &mut BufferedInput, auth: &str) {
    if auth.is_empty() {
        return;
    }
    write!(input, "{auth}\r\n").unwrap();
}

#[derive(Debug)]
pub struct Transformer {
    config: TransformerConfig,

    phase: Phase,
    in_paragraph: bool,
    ignore_next_newline: bool,

    mxp_active: bool,
    mxp_entity_string: Vec<u8>,
    mxp_mode_default: mxp::Mode,
    mxp_mode_previous: mxp::Mode,
    mxp_mode: mxp::Mode,
    mxp_quote_terminator: Option<NonZeroU8>,
    mxp_state: mxp::State,
    mxp_tags: TagList,

    charsets: charset::Charsets,
    decompress: mccp::Decompress,
    mnes_variables: mnes::Variables,
    ttype_negotiator: mtts::Negotiator,

    ansi: ansi::Interpreter,
    subnegotiation_data: Vec<u8>,
    subnegotiation_type: u8,
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
    pub fn new(config: TransformerConfig) -> Self {
        let mut output = BufferedOutput::new();
        output.set_colors(config.colors.clone());
        if config.ignore_mxp_colors {
            output.disable_mxp_colors();
        }
        let mut mxp_state = mxp::State::new();
        if config.use_mxp == UseMxp::Always {
            mxp_state.add_globals();
        }
        Self {
            phase: Phase::Normal,

            mxp_active: config.use_mxp == UseMxp::Always,

            in_paragraph: false,
            ignore_next_newline: false,
            mxp_mode_default: mxp::Mode::OPEN,
            mxp_mode: mxp::Mode::OPEN,
            mxp_mode_previous: mxp::Mode::OPEN,
            mxp_quote_terminator: None,
            mxp_entity_string: Vec::new(),
            mxp_tags: TagList::new(),
            mxp_state,

            ansi: ansi::Interpreter::new(),
            charsets: charset::Charsets::new(),
            decompress: mccp::Decompress::new(),
            mnes_variables: mnes::Variables::new(),
            ttype_negotiator: mtts::Negotiator::new(),

            subnegotiation_type: 0,
            subnegotiation_data: Vec::new(),

            utf8_sequence: Vec::with_capacity(4),
            output,
            input: BufferedInput::new(),

            config,
        }
    }

    pub fn subnegotiate<T: Negotiate>(&mut self, negotiator: T) {
        self.input.append([telnet::IAC, telnet::SB, T::CODE]);
        let subnegotiation = negotiator.negotiate(&self.config);
        write!(self.input, "{subnegotiation}").unwrap();
        self.input.append([telnet::IAC, telnet::SE]);
    }

    pub fn set_config(&mut self, mut config: TransformerConfig) {
        mem::swap(&mut self.config, &mut config);
        if self.config.ignore_mxp_colors {
            self.output.disable_mxp_colors();
        } else {
            self.output.enable_mxp_colors();
        }
        if config.colors != self.config.colors {
            self.output.set_colors(self.config.colors.clone());
        }
        match self.config.use_mxp {
            UseMxp::Always => self.mxp_on(),
            UseMxp::Never => self.mxp_off(true),
            UseMxp::Command | UseMxp::Query => (),
        }
        let mnes_updates = self.mnes_variables.changes(&config, &self.config);
        if mnes_updates.is_empty() {
            return;
        }
        self.subnegotiate(mnes_updates);
    }

    pub fn has_output(&self) -> bool {
        !self.output.is_empty()
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
        self.output.append(err);
    }

    fn take_mxp_string(&mut self) -> mxp::Result<String> {
        String::from_utf8(mem::take(&mut self.mxp_entity_string)).map_err(|e| {
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
        self.mxp_close_tags_from(0);
        self.output.append(TelnetFragment::Mxp { enabled: false });

        if !completely {
            return;
        }
        self.mxp_mode_change(Some(mxp::Mode::OPEN));
        if self.phase.is_mxp() {
            self.phase = Phase::Normal;
        }
        self.mxp_active = false;
    }

    fn mxp_on(&mut self) {
        if self.mxp_active {
            return;
        }
        self.output.append(TelnetFragment::Mxp { enabled: true });
        self.mxp_active = true;
        self.mxp_mode_default = mxp::Mode::OPEN;
        self.mxp_mode = mxp::Mode::OPEN;
        self.mxp_tags.clear();
        self.mxp_state.clear();
        self.mxp_state.add_globals();
    }

    fn mxp_endtag(&mut self, tag_body: &str) -> mxp::Result<()> {
        let was_secure = self.mxp_mode.is_secure();
        self.mxp_restore_mode();
        let name = Tag::parse_closing_tag(tag_body)?;
        let (closed, _tag) = self.mxp_tags.find_last(was_secure, name)?;
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
        let Some(entity) = self.mxp_state.define(tag)? else {
            return Ok(());
        };
        self.output.append(EntityFragment::entity(&entity));
        Ok(())
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
        if !component.is_command() {
            let tag = Tag::new(component, secure, self.output.span_len())?;
            self.mxp_tags.push(tag);
        }

        let mut args = mxp::Arguments::parse(words)?;

        match component {
            mxp::ElementComponent::Atom(atom) => {
                let scanner = mxp_state.decode_args(&mut args);
                self.mxp_open_atom(mxp::Action::new(atom.action, scanner)?, mxp_state);
            }
            mxp::ElementComponent::Custom(el) => {
                if let Some(variable) = &el.variable {
                    self.output.set_mxp_entity(EntitySetter {
                        name: variable.clone(),
                        flags: EnumSet::new(),
                        is_variable: true,
                    });
                }
                self.mxp_open_element(el, &args, mxp_state)?;
            }
        }

        Ok(())
    }

    fn mxp_set_entity(
        &mut self,
        variable: String,
        keywords: EnumSet<mxp::EntityKeyword>,
        mxp_state: &mxp::State,
    ) {
        if mxp_state.is_global_entity(&variable) {
            self.handle_mxp_error(mxp::Error::new(
                variable,
                mxp::ErrorKind::CannotRedefineEntity,
            ));
            return;
        }
        self.output.set_mxp_entity(EntitySetter {
            name: variable,
            flags: keywords,
            is_variable: false,
        });
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
            self.mxp_open_atom(action?, mxp_state);
        }
        if let Some(fore) = el.fore {
            self.output.set_mxp_foreground(fore);
        }
        if let Some(back) = el.back {
            self.output.set_mxp_background(back);
        }
        Ok(())
    }

    fn mxp_open_atom(&mut self, action: mxp::Action<Cow<str>>, mxp_state: &mxp::State) {
        use mxp::Action;

        match action {
            Action::Bold => self.output.set_mxp_flag(TextStyle::Bold),
            Action::Br => self.output.start_line(),
            Action::Color { fore, back } => {
                if let Some(fg) = fore {
                    self.output.set_mxp_foreground(fg);
                }
                if let Some(bg) = back {
                    self.output.set_mxp_background(bg);
                }
            }
            Action::Dest { name } => self.output.set_mxp_window(name.into_owned()),
            Action::Expire { name } => self
                .output
                .append(EffectFragment::ExpireLinks(name.map(Cow::into_owned))),
            Action::Filter(filter) => self.output.append(filter.into_owned()),
            Action::Font(font) => self.output.set_mxp_font(font.into_owned()),
            Action::Frame(frame) => self.output.append(frame.into_owned()),
            Action::Gauge(gauge) => self.output.append(gauge.into_owned()),
            Action::Heading(heading) => self.output.set_mxp_heading(heading),
            Action::Highlight => self.output.set_mxp_flag(TextStyle::Highlight),
            Action::Hr => self.output.append(OutputFragment::Hr),
            Action::Image(image) => self.output.append(image.into_owned()),
            Action::Italic => self.output.set_mxp_flag(TextStyle::Italic),
            Action::Link(link) => self.output.set_mxp_action(link.clone()),
            Action::Music(music) => self.output.append(music.into_owned()),
            Action::MusicOff => self.output.append(EffectFragment::MusicOff),
            Action::Mxp { keywords } => self.mxp_set_keywords(keywords),
            Action::NoBr => self.ignore_next_newline = true,
            Action::P => self.in_paragraph = true,
            Action::Password => input_mxp_auth(&mut self.input, &self.config.password),
            Action::Relocate(relocate) => self.output.append(relocate.into_owned()),
            Action::Reset => self.mxp_off(false),
            Action::SBr => self.output.push(b' '),
            Action::Small => self.output.set_mxp_flag(TextStyle::Small),
            Action::Sound(sound) => self.output.append(sound.into_owned()),
            Action::SoundOff => self.output.append(EffectFragment::SoundOff),
            Action::Stat(stat) => self.output.append(stat.into_owned()),
            Action::Strikeout => self.output.set_mxp_flag(TextStyle::Strikeout),
            Action::Support { questions } => {
                let supported_actions = self.config.supported_actions();
                mxp::Atom::fmt_supported(self.input.as_mut(), questions, supported_actions);
            }
            Action::Tt => self.output.set_mxp_flag(TextStyle::NonProportional),
            Action::Underline => self.output.set_mxp_flag(TextStyle::Underline),
            Action::User => input_mxp_auth(&mut self.input, &self.config.player),
            Action::Var { variable, keywords } => {
                self.mxp_set_entity(variable.into_owned(), keywords, mxp_state);
            }
            Action::Version => {
                let response = mxp::responses::IdentifyResponse {
                    name: &self.config.app_name,
                    version: &self.config.version,
                };
                write!(self.input, "{response}").unwrap();
            }
        }
    }

    fn mxp_set_keywords(&mut self, keywords: EnumSet<mxp::MxpKeyword>) {
        use mxp::{Mode, MxpKeyword};
        if keywords.contains(MxpKeyword::Off) {
            self.mxp_off(true);
        }
        if keywords.contains(MxpKeyword::DefaultLocked) {
            self.mxp_mode_default = Mode::LOCKED;
        } else if keywords.contains(MxpKeyword::DefaultSecure) {
            self.mxp_mode_default = Mode::SECURE;
        } else if keywords.contains(MxpKeyword::DefaultOpen) {
            self.mxp_mode_default = Mode::OPEN;
        }

        if keywords.contains(MxpKeyword::IgnoreNewlines) {
            self.in_paragraph = true;
        } else if keywords.contains(MxpKeyword::UseNewlines) {
            self.in_paragraph = false;
        }
    }

    fn mxp_close_tags_from(&mut self, pos: usize) {
        if let Some(span_index) = self.mxp_tags.truncate(pos) {
            self.output
                .truncate_spans(span_index, self.mxp_state.entities_mut());
        }
    }

    fn mxp_collected_entity(&mut self) -> mxp::Result<()> {
        let mxp_string = self.take_mxp_string()?;
        let name = mxp_string.trim();
        mxp::validate(name, mxp::ErrorKind::InvalidEntityName)?;
        if let Some(entity) = self.mxp_state.get_entity(name)? {
            self.mxp_active = false;
            self.output.append_text(entity);
            self.mxp_active = true;
        }
        Ok(())
    }

    fn mxp_mode_change(&mut self, newmode: Option<mxp::Mode>) {
        let oldmode = self.mxp_mode;
        let newmode = newmode.unwrap_or(self.mxp_mode_default);
        let closing = oldmode.is_open() && !newmode.is_open();
        if closing {
            let closed = self.mxp_tags.last_unsecure_index();
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

    #[inline]
    pub fn receive(&mut self, bytes: &[u8], buf: &mut [u8]) -> io::Result<()> {
        if bytes.is_empty() {
            return Ok(());
        }
        let mut cursor = ReceiveCursor::new(bytes);
        if !self.decompress.active() {
            for byte in &mut cursor {
                self.receive_byte(byte);
                if self.decompress.active() {
                    break;
                }
            }
        }
        while !cursor.is_empty() {
            let n = self.decompress.decompress(&mut cursor, buf)?;
            let mut iter = buf[..n].iter();
            for &byte in &mut iter {
                self.receive_byte(byte);
                if !self.decompress.active() {
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
            self.output.append(EffectFragment::CarriageReturn);
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
                self.phase = Phase::Ansi;
                self.ansi.reset();
            }
            Phase::Esc => self.phase = Phase::Normal,

            Phase::Utf8Character => self.utf8_sequence.push(c),

            Phase::Ansi => match self.ansi.interpret(c, &mut self.output) {
                ansi::Outcome::Continue => (),
                ansi::Outcome::Done => self.phase = Phase::Normal,
                ansi::Outcome::Mxp(mxp::Mode::RESET) => {
                    self.mxp_off(false);
                    self.phase = Phase::Normal;
                }
                ansi::Outcome::Mxp(mode) => {
                    self.mxp_mode_change(Some(mode));
                    self.phase = Phase::Normal;
                }
            },

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
                        self.output.append(EffectFragment::EraseCharacter);
                    }
                    telnet::EL => {
                        self.phase = Phase::Normal;
                        self.output.append(EffectFragment::EraseLine);
                    }
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
                let supported = match c {
                    protocol::MCCP1 | protocol::MCCP2 => {
                        !self.config.disable_compression && self.decompress.will(c)
                    }
                    protocol::SGA
                    | protocol::MUD_SPECIFIC
                    | protocol::CHARSET
                    | protocol::MNES
                    | protocol::MSSP => true,
                    protocol::ECHO if self.config.no_echo_off => false,
                    protocol::ECHO => {
                        self.output
                            .append(TelnetFragment::SetEcho { should_echo: false });
                        true
                    }
                    protocol::MXP => match self.config.use_mxp {
                        UseMxp::Never => false,
                        UseMxp::Always | UseMxp::Command => true,
                        UseMxp::Query => {
                            self.mxp_on();
                            true
                        }
                    },
                    telnet::WILL_EOR => true,
                    _ if self.config.will.contains(&c) => true,
                    _ => false,
                };
                self.input.append(telnet::supports_do(c, supported));
                self.output.append(TelnetFragment::Negotiation {
                    source: TelnetSource::Client,
                    verb: if supported {
                        TelnetVerb::Do
                    } else {
                        TelnetVerb::Dont
                    },
                    code: c,
                });
            }

            Phase::Wont => {
                self.phase = Phase::Normal;
                self.output.append(TelnetFragment::Negotiation {
                    source: TelnetSource::Server,
                    verb: TelnetVerb::Wont,
                    code: c,
                });
                if c == protocol::ECHO && !self.config.no_echo_off {
                    self.output
                        .append(TelnetFragment::SetEcho { should_echo: true });
                }
                self.input.append(telnet::supports_do(c, false));
                self.output.append(TelnetFragment::Negotiation {
                    source: TelnetSource::Client,
                    code: c,
                    verb: TelnetVerb::Dont,
                });
            }

            Phase::Do => {
                self.phase = Phase::Normal;
                self.output.append(TelnetFragment::Negotiation {
                    source: TelnetSource::Server,
                    verb: TelnetVerb::Do,
                    code: c,
                });
                let supported = match c {
                    protocol::SGA
                    | protocol::MUD_SPECIFIC
                    | protocol::ECHO
                    | protocol::CHARSET
                    | protocol::MSSP
                    | protocol::MNES => true,
                    protocol::MTTS => {
                        self.ttype_negotiator.reset();
                        true
                    }
                    protocol::NAWS => self.config.naws,
                    protocol::MXP => match self.config.use_mxp {
                        UseMxp::Never => false,
                        UseMxp::Always | UseMxp::Command => true,
                        UseMxp::Query => {
                            self.mxp_on();
                            true
                        }
                    },
                    _ if self.config.will.contains(&c) => true,
                    _ => false,
                };
                self.input.append(telnet::supports_will(c, supported));
                self.output.append(TelnetFragment::Negotiation {
                    source: TelnetSource::Client,
                    verb: if supported {
                        TelnetVerb::Will
                    } else {
                        TelnetVerb::Wont
                    },
                    code: c,
                });
                if c == protocol::NAWS && supported {
                    self.output.append(TelnetFragment::Naws);
                }
            }

            Phase::Dont => {
                self.output.append(TelnetFragment::Negotiation {
                    source: TelnetSource::Server,
                    verb: TelnetVerb::Dont,
                    code: c,
                });
                self.phase = Phase::Normal;
                match c {
                    protocol::MXP if self.mxp_active => self.mxp_off(true),
                    protocol::MTTS => self.ttype_negotiator.reset(),
                    protocol::MNES => self.mnes_variables.clear(),
                    _ => (),
                }
                self.input.append(telnet::supports_will(c, false));
                self.output.append(TelnetFragment::Negotiation {
                    source: TelnetSource::Client,
                    verb: TelnetVerb::Wont,
                    code: c,
                });
            }

            Phase::Sb if c == protocol::MCCP1 => self.phase = Phase::Compress,
            Phase::Sb => {
                self.subnegotiation_type = c;
                self.subnegotiation_data.clear();
                self.phase = Phase::Subnegotiation;
            }

            Phase::Subnegotiation if c == telnet::IAC => self.phase = Phase::SubnegotiationIac,
            Phase::Subnegotiation => self.subnegotiation_data.push(c),

            Phase::Compress if c == telnet::WILL => self.phase = Phase::CompressWill,
            Phase::Compress => self.phase = Phase::Normal,

            Phase::CompressWill if c == telnet::SE => self.decompress.set_active(true),
            Phase::CompressWill => self.phase = Phase::Normal,

            Phase::SubnegotiationIac if c == telnet::IAC => {
                self.subnegotiation_data.push(c);
                self.phase = Phase::Subnegotiation;
            }
            Phase::SubnegotiationIac => {
                self.phase = Phase::Normal;
                match self.subnegotiation_type {
                    protocol::MCCP2 => {
                        if !self.config.disable_compression {
                            self.decompress.set_active(true);
                        }
                    }
                    protocol::MXP => {
                        if self.config.use_mxp == UseMxp::Command {
                            self.mxp_on();
                        }
                    }
                    protocol::MTTS if !self.config.terminal_identification.is_empty() => {
                        if self.subnegotiation_data.first() == Some(&mtts::SEND) {
                            self.subnegotiate(self.ttype_negotiator);
                            self.ttype_negotiator.advance();
                        }
                    }
                    protocol::CHARSET => {
                        self.charsets = charset::Charsets::from(&self.subnegotiation_data);
                        self.subnegotiate(self.charsets);
                    }
                    protocol::MSSP => {
                        for (variable, value) in mssp::iter(&self.subnegotiation_data) {
                            self.output.append_server_status(variable, value);
                        }
                    }
                    protocol::MNES => {
                        self.mnes_variables = mnes::Variables::from(&self.subnegotiation_data);
                        self.subnegotiate(self.mnes_variables);
                    }
                    _ => (),
                }
                self.output
                    .append_subnegotiation(self.subnegotiation_type, &self.subnegotiation_data);
                self.subnegotiation_data.clear();
            }

            Phase::MxpElement => match c {
                b'>' => {
                    if let Err(e) = self.mxp_collected_element() {
                        self.handle_mxp_error(e);
                    }
                    self.phase = Phase::Normal;
                }
                b'<' => {
                    self.handle_mxp_error(mxp::Error::new(
                        &self.mxp_entity_string,
                        mxp::ErrorKind::UnterminatedElement,
                    ));
                    self.mxp_entity_string.clear();
                    self.mxp_entity_string.push(c);
                }
                b'\'' => {
                    const_non_zero!(NON_ZERO_APOSTROPHE, NonZeroU8, b'\'');
                    self.mxp_entity_string.push(c);
                    self.mxp_quote_terminator = Some(NON_ZERO_APOSTROPHE);
                    self.phase = Phase::MxpQuote;
                }
                b'"' => {
                    const_non_zero!(NON_ZERO_QUOTE, NonZeroU8, b'"');
                    self.mxp_entity_string.push(c);
                    self.mxp_quote_terminator = Some(NON_ZERO_QUOTE);
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

            Phase::MxpComment if c == b'>' && self.mxp_entity_string.ends_with(b"--") => {
                self.phase = Phase::Normal;
            }

            Phase::MxpComment => self.mxp_entity_string.push(c),

            Phase::MxpQuote => {
                if let Some(terminator) = self.mxp_quote_terminator {
                    if terminator.get() == c {
                        self.phase = Phase::MxpElement;
                        self.mxp_quote_terminator = None;
                    }
                }
                self.mxp_entity_string.push(c);
            }

            Phase::MxpEntity => match c {
                b';' => {
                    self.phase = Phase::Normal;
                    if let Err(e) = self.mxp_collected_entity() {
                        self.handle_mxp_error(e);
                    }
                }
                b'&' => {
                    self.mxp_entity_string.push(c);
                    self.handle_mxp_error(mxp::Error::new(
                        &self.mxp_entity_string,
                        mxp::ErrorKind::UnterminatedEntity,
                    ));
                    self.mxp_entity_string.clear();
                }
                b'<' => {
                    self.mxp_entity_string.push(c);
                    self.handle_mxp_error(mxp::Error::new(
                        &self.mxp_entity_string,
                        mxp::ErrorKind::UnterminatedEntity,
                    ));
                    self.mxp_entity_string.clear();
                    self.phase = Phase::MxpElement;
                }
                _ => self.mxp_entity_string.push(c),
            },

            Phase::MxpWelcome => (),

            Phase::Normal => match c {
                telnet::ESC => self.phase = Phase::Esc,
                telnet::IAC => self.phase = Phase::Iac,
                // BEL
                0x07 => self.output.append(EffectFragment::Beep),
                // BS
                0x08 => self.output.append(EffectFragment::Backspace),
                // FF
                0x0C => self.output.append(OutputFragment::PageBreak),
                b'\t' if self.in_paragraph => {
                    if last_char != b' ' {
                        self.output.append_text(" ");
                    }
                }
                b'\r' => (),
                b' ' if last_char == b' ' && self.in_paragraph => {}
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
                            b'.' => self.output.append_text("  "),
                            b' ' | b'\t' | 0x0C => (),
                            _ => self.output.append_text(" "),
                        }
                    } else if self.ignore_next_newline {
                        self.ignore_next_newline = false;
                    } else {
                        self.output.start_line();
                    }
                }
                _ if is_utf8_higher_order(c) => {
                    self.utf8_sequence.push(c);
                    self.phase = Phase::Utf8Character;
                }
                _ if !self.mxp_active || !self.mxp_mode.is_mxp() => self.output.push(c),
                b'<' => {
                    self.mxp_entity_string.clear();
                    self.phase = Phase::MxpElement;
                }
                b'&' => {
                    self.mxp_entity_string.clear();
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
