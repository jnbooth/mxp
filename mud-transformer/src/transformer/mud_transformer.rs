use std::borrow::Cow;
use std::num::NonZero;
use std::{io, mem, slice};

use bytes::BytesMut;
use flagset::FlagSet;
use mxp::escape::{ansi, telnet};
use mxp::responses::{SupportResponse, VersionResponse};

use super::config::{TabBehavior, TransformerConfig, UseMxp};
use super::cursor::ReceiveCursor;
use super::phase::Phase;
use super::state::StateLock;
use super::tag::{Tag, TagList};
use crate::input::{BufferedInput, Drain as InputDrain};
use crate::output::{
    BufferedOutput, ControlFragment, EntityFragment, EntitySetter, MxpFragment, OutputDrain,
    OutputFragment, TelnetFragment, TelnetSource, TelnetVerb, TextStyle,
};
use crate::protocol::{self, Negotiate, charset, mccp, mnes, mssp, mtts, xterm};
use crate::term::{CursorEffect, EraseRange, EraseTarget};

fn input_mxp_auth(input: &mut BufferedInput, auth: &str) {
    if auth.is_empty() {
        return;
    }
    writeln!(input, "{auth}\r");
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
    mxp_quote_terminator: Option<NonZero<u8>>,
    mxp_state: StateLock,
    mxp_tags: TagList,

    charsets: charset::Charsets,
    decompress: mccp::Decompress,
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
    pub fn new(config: TransformerConfig) -> Self {
        let mut output = BufferedOutput::new();
        output.set_colors(&config.colors);
        if config.ignore_mxp_colors {
            output.disable_mxp_colors();
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
            mxp_state: mxp::State::populated().into(),

            charsets: charset::Charsets::new(),
            decompress: mccp::Decompress::new(),
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
        self.decompress.active()
    }

    pub fn mxp_active(&self) -> bool {
        self.mxp_active
    }

    fn subnegotiate<T: Negotiate>(&mut self, negotiator: T) {
        self.input.append([telnet::IAC, telnet::SB, T::CODE]);
        negotiator.negotiate(&mut self.input, &self.config).unwrap();
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
            self.output.set_colors(&self.config.colors);
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

    pub fn published_entities(&self) -> mxp::PublishedIter<'_> {
        self.mxp_state.published_entities()
    }

    pub fn published_variables(&self) -> mxp::PublishedIter<'_> {
        self.output.published_variables()
    }

    pub fn count_custom_mxp_elements(&self) -> usize {
        self.mxp_state.count_custom_elements()
    }

    pub fn count_custom_mxp_entities(&self) -> usize {
        self.mxp_state.count_custom_entities()
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
    }

    fn mxp_endtag(&mut self, tag_body: &str) -> mxp::Result<()> {
        let was_secure = self.mxp_mode.is_secure();
        self.mxp_restore_mode();
        let name = Tag::parse_closing_tag(tag_body)?;
        let (closed, _tag) = self.mxp_tags.find_last(was_secure, name)?;
        self.mxp_close_tags_from(closed);
        Ok(())
    }

    fn mxp_definition(
        &mut self,
        definition: mxp::CollectedDefinition,
        text: &str,
    ) -> mxp::Result<()> {
        let was_secure = self.mxp_mode.is_secure();
        self.mxp_restore_mode();
        if !was_secure {
            return Err(mxp::Error::new(
                text,
                mxp::ErrorKind::DefinitionWhenNotSecure,
            ));
        }
        let Some(entity) = self.mxp_state.define(definition)? else {
            return Ok(());
        };
        self.output.append(EntityFragment::entity(&entity));
        Ok(())
    }

    fn mxp_collected_element(&mut self) -> mxp::Result<()> {
        let text = self.take_mxp_string()?;
        match mxp::Element::collect(&text)? {
            mxp::CollectedElement::Definition(definition) => self.mxp_definition(definition, &text),
            mxp::CollectedElement::TagClose(text) => self.mxp_endtag(text),
            mxp::CollectedElement::TagOpen(text) => {
                let mxp_state = self.mxp_state.take();
                let result = self.mxp_start_tag(text, &mxp_state);
                self.mxp_state.set(mxp_state);
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

        match component {
            mxp::ElementComponent::Tag(tag) => {
                let args = words.parse_args::<&str>()?;
                self.mxp_open_tag(mxp_state.decode_tag(tag, &args)?, mxp_state);
            }
            mxp::ElementComponent::Element(el) => {
                if let Some(variable) = &el.variable {
                    self.output.set_mxp_entity(EntitySetter {
                        name: variable.clone(),
                        flags: FlagSet::default(),
                        is_variable: true,
                    });
                }
                let args = words.parse_args::<&str>()?;
                self.mxp_open_element(el, &args, mxp_state)?;
            }
        }

        Ok(())
    }

    fn mxp_set_entity(
        &mut self,
        variable: String,
        keywords: FlagSet<mxp::EntityKeyword>,
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

    fn mxp_open_element<S: AsRef<str>>(
        &mut self,
        el: &mxp::Element,
        args: &mxp::Arguments<S>,
        mxp_state: &mxp::State,
    ) -> mxp::Result<()> {
        if el.gag {
            self.output.set_mxp_gag();
        }
        if let Some(window) = &el.window {
            self.output.set_mxp_window(window);
        }
        for action in mxp_state.decode_element(el, args) {
            self.mxp_open_tag(action?, mxp_state);
        }
        if let Some(fore) = el.fore {
            self.output.set_mxp_foreground(fore);
        }
        if let Some(back) = el.back {
            self.output.set_mxp_background(back);
        }
        Ok(())
    }

    fn mxp_open_tag(&mut self, action: mxp::Action<Cow<str>>, mxp_state: &mxp::State) {
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
            Action::Dest { name } => self.output.set_mxp_window(&name),
            Action::Expire { name } => self
                .output
                .append(MxpFragment::ExpireLinks(name.map(Cow::into_owned))),
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
            Action::MusicOff => self.output.append(MxpFragment::MusicOff),
            Action::Mxp { keywords } => self.mxp_set_keywords(keywords),
            Action::NoBr => self.ignore_next_newline = true,
            Action::P => self.in_paragraph = true,
            Action::Password => input_mxp_auth(&mut self.input, &self.config.password),
            Action::Relocate(relocate) => self.output.append(relocate.into_owned()),
            Action::Reset => self.mxp_off(false),
            Action::SBr => self.output.append_text(" "),
            Action::Small => self.output.set_mxp_flag(TextStyle::Small),
            Action::Sound(sound) => self.output.append(sound.into_owned()),
            Action::SoundOff => self.output.append(MxpFragment::SoundOff),
            Action::Stat(stat) => self.output.append(stat.into_owned()),
            Action::Strikeout => self.output.set_mxp_flag(TextStyle::Strikeout),
            Action::Support { questions } => {
                let supported_actions = self.config.supported_actions();
                let supported_tags = SupportResponse::new(&questions, supported_actions);
                writeln!(self.input, "{supported_tags}\r");
            }
            Action::Tt => self.output.set_mxp_flag(TextStyle::NonProportional),
            Action::Underline => self.output.set_mxp_flag(TextStyle::Underline),
            Action::User => input_mxp_auth(&mut self.input, &self.config.player),
            Action::Var { variable, keywords } => {
                self.mxp_set_entity(variable.into_owned(), keywords, mxp_state);
            }
            Action::Version => {
                let response = VersionResponse {
                    name: &self.config.app_name,
                    version: &self.config.version,
                };
                writeln!(self.input, "{response}\r");
            }
        }
    }

    fn mxp_set_keywords(&mut self, keywords: FlagSet<mxp::MxpKeyword>) {
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
        match self.mxp_state.decode_entity(name)? {
            Some(mxp::DecodedEntity::Char(c)) => self.output.append_char(c),
            Some(mxp::DecodedEntity::Str(s)) => self.output.append_text(s),
            None => (),
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
        let mxp_state = self.mxp_state.take();
        if let Some(element) = mxp_state.get_line_tag(newmode)
            && let Err(e) =
                self.mxp_open_element(element, &mxp::Arguments::<&str>::new(), &mxp_state)
        {
            self.handle_mxp_error(e);
        }
        self.mxp_state.set(mxp_state);
    }

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
        if self.after_ansi
            && c != ansi::ESC
            && !matches!(self.phase, Phase::Esc | Phase::Ansi | Phase::AnsiString)
        {
            self.after_ansi = false;
            self.ansi.clear_mslp_link();
        }

        if self.last_char == b'\r' && c != b'\n' {
            self.output.append(ControlFragment::CarriageReturn);
        }

        if self.phase == Phase::Utf8Character && !is_utf8_continuation(c) {
            let sequence = str::from_utf8(&self.utf8_sequence).unwrap_or("�");
            self.output.append_text(sequence);
            self.phase = Phase::Normal;
        }

        if self.phase.is_phase_reset(c) {
            self.phase = Phase::Normal;
            if matches!(self.phase, Phase::Ansi | Phase::AnsiString) {
                self.ansi.terminate();
            }
        }

        match self.phase {
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
                    xterm::Outcome::Done | xterm::Outcome::Fail => (),
                    xterm::Outcome::Link => {
                        if let Some(link) = self.ansi.take_mslp_link() {
                            self.output.set_mxp_action(link);
                        } else if self.config.linkify_underlined {
                            self.output.set_mxp_action(mxp::Link::for_text());
                        }
                    }
                    xterm::Outcome::Mxp(mxp::Mode::RESET) => self.mxp_off(false),
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
                    protocol::MCCP2 => !self.config.disable_compression,
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
                        if data.first() == Some(&mtts::SEND) {
                            self.subnegotiate(self.ttype_negotiator);
                            self.ttype_negotiator.advance();
                        }
                    }
                    protocol::CHARSET => {
                        self.charsets = charset::Charsets::from(&data);
                        self.subnegotiate(self.charsets);
                    }
                    protocol::MSSP => {
                        for (variable, value) in mssp::iter(&data) {
                            self.output
                                .append(TelnetFragment::ServerStatus { variable, value });
                        }
                    }
                    protocol::MNES => {
                        self.mnes_variables = mnes::Variables::from(&data);
                        self.subnegotiate(self.mnes_variables);
                    }
                    _ => (),
                }
                self.output.append(TelnetFragment::Subnegotiation {
                    code: self.subnegotiation_type,
                    data,
                });
            }

            Phase::MxpElement | Phase::MxpComment | Phase::MxpQuote | Phase::MxpEntity
                if c == ansi::ESC =>
            {
                self.handle_mxp_error(mxp::Error::new(
                    &self.mxp_entity_string,
                    mxp::ErrorKind::UnterminatedElement,
                ));
                self.mxp_entity_string.clear();
                self.phase = Phase::Esc;
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
                    self.mxp_entity_string.push(c);
                    self.mxp_quote_terminator = NonZero::new(b'\'');
                    self.phase = Phase::MxpQuote;
                }
                b'"' => {
                    self.mxp_entity_string.push(c);
                    self.mxp_quote_terminator = NonZero::new(b'"');
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
                if let Some(terminator) = self.mxp_quote_terminator
                    && terminator.get() == c
                {
                    self.phase = Phase::MxpElement;
                    self.mxp_quote_terminator = None;
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

            Phase::Normal => {
                let last_char = self.last_char;
                self.last_char = c;
                match c {
                    telnet::IAC => self.phase = Phase::Iac,
                    ansi::ESC => self.phase = Phase::Esc,
                    ansi::ENQ => self.input.append(self.ansi.answerback()),
                    ansi::BEL => self.output.append(ControlFragment::Beep),
                    ansi::BS => self.output.append(CursorEffect::Back(1)),
                    ansi::VT => self.output.append(ControlFragment::VerticalTab),
                    ansi::FF => self.output.append(OutputFragment::PageBreak),
                    b'\r' => (),
                    b'\t' if self.in_paragraph => {
                        if last_char != b' ' {
                            self.output.append_text(" ");
                        }
                    }
                    b'\t' => match self.config.tab {
                        TabBehavior::Control => self.output.append(CursorEffect::TabForward(1)),
                        TabBehavior::NextMultipleOf8 => self.output.append_tab(),
                        tab => self.output.append_text(tab.string()),
                    },
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
                    b'<' if self.mxp_active && self.mxp_mode.is_mxp() => {
                        self.mxp_entity_string.clear();
                        self.phase = Phase::MxpElement;
                    }
                    b'&' if self.mxp_active && self.mxp_mode.is_mxp() => {
                        self.mxp_entity_string.clear();
                        self.phase = Phase::MxpEntity;
                    }
                    32..=126 => self.output.append_text(
                        // SAFETY: `utf8` is valid UTF-8, since it is a single ASCII byte.
                        unsafe { str::from_utf8_unchecked(slice::from_ref(&c)) },
                    ),
                    128.. => {
                        self.utf8_sequence.push(c);
                        self.phase = Phase::Utf8Character;
                    }
                    ..32 | ansi::DEL => (),
                }
            }
        }
    }
}

#[inline]
pub const fn is_utf8_continuation(c: u8) -> bool {
    (c & 0xC0) != 0x80
}
