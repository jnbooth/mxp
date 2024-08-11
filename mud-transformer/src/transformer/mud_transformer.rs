use std::{io, mem};

use super::config::{TransformerConfig, UseMxp};
use super::input::{BufferedInput, Drain as InputDrain};
use super::phase::Phase;
use super::tag::{Tag, TagList};
use crate::output::{BufferedOutput, Heading, InList, OutputDrain, TextFormat, TextStyle};
use crate::receive::{Decompress, ReceiveCursor};
use crate::EffectFragment;
use mxp::escape::{ansi, telnet};
use mxp::{RgbColor, WorldColor};

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
    mxp_quote_terminator: Option<u8>,
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

    fn handle_mxp_error(&self, err: mxp::ParseError) {
        eprintln!("MXP Error: {}", err);
    }

    fn take_mxp_string(&mut self) -> Result<String, mxp::ParseError> {
        String::from_utf8(mem::take(&mut self.mxp_string)).map_err(|e| {
            let bytes_debug = format!("{:?}", e.as_bytes());
            mxp::ParseError::new(bytes_debug, mxp::Error::MalformedBytes)
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

    fn mxp_endtag(&mut self, tag_body: &str) -> Result<(), mxp::ParseError> {
        let was_secure = self.mxp_mode.is_secure();
        self.mxp_restore_mode();
        let name = Tag::parse_closing_tag(tag_body)?;
        let (closed, _tag) = self.mxp_active_tags.find_last(was_secure, name)?;
        self.mxp_close_tags_from(closed);
        Ok(())
    }

    fn mxp_definition(&mut self, tag: &str) -> Result<(), mxp::ParseError> {
        let was_secure = self.mxp_mode.is_secure();
        self.mxp_restore_mode();
        if !was_secure {
            return Err(mxp::ParseError::new(
                tag,
                mxp::Error::DefinitionWhenNotSecure,
            ));
        }
        self.mxp_state.define(tag)
    }

    fn mxp_collected_element(&mut self) -> Result<(), mxp::ParseError> {
        match mxp::Element::collect(&self.take_mxp_string()?)? {
            mxp::CollectedElement::Definition(text) => self.mxp_definition(text),
            mxp::CollectedElement::TagClose(text) => self.mxp_endtag(text),
            mxp::CollectedElement::TagOpen(text) => self.mxp_start_tag(text),
        }
    }

    fn mxp_start_tag(&mut self, tag: &str) -> Result<(), mxp::ParseError> {
        let secure = self.mxp_mode.is_secure();
        self.mxp_restore_mode();
        let mut words = mxp::Words::new(tag);
        let name = words.validate_next_or(mxp::Error::InvalidElementName)?;
        let component = self.mxp_state.get_component(name)?;
        let tag = Tag::new(component, secure, self.output.span_len())?;
        self.mxp_active_tags.push(tag);

        if let Some(variable) = component.variable() {
            self.output.set_mxp_variable(variable);
        }

        let mut args = mxp::Arguments::parse_words(words)?;

        match component {
            mxp::ElementComponent::Atom(atom) => {
                self.mxp_state.decode_args(&mut args)?;
                self.mxp_open_atom(atom.action, args);
            }
            mxp::ElementComponent::Custom(el) => {
                // create a temporary vector to avoid borrow conflict
                // could clone the element instead, but that seems like a waste
                let actions: Result<Vec<_>, mxp::ParseError> =
                    self.mxp_state.decode_element(el, &args).collect();
                for (action, newargs) in actions? {
                    self.mxp_open_atom(action, newargs);
                }
            }
        }

        Ok(())
    }

    fn mxp_open_atom(&mut self, mut action: mxp::Action, args: mxp::Arguments) {
        use mxp::{Action, Atom, Keyword, Link, SendTo};
        const SPECIAL_LINK: &str = "&text;";
        if action == Action::Hyperlink && args.get("xch_cmd").is_some() {
            self.pueblo_active = true;
            action = Action::Send;
        }
        match action {
            Action::H1 => self.output.set_mxp_heading(Heading::H1),
            Action::H2 => self.output.set_mxp_heading(Heading::H2),
            Action::H3 => self.output.set_mxp_heading(Heading::H3),
            Action::H4 => self.output.set_mxp_heading(Heading::H4),
            Action::H5 => self.output.set_mxp_heading(Heading::H5),
            Action::H6 => self.output.set_mxp_heading(Heading::H5),
            Action::Bold => self.output.set_mxp_flag(TextStyle::Bold),
            Action::Underline => self.output.set_mxp_flag(TextStyle::Underline),
            Action::Italic => self.output.set_mxp_flag(TextStyle::Italic),
            Action::Color => {
                let mxp::ColorArgs { fore, back } = (&args).into();
                if let Some(fg) = fore.and_then(RgbColor::named) {
                    self.output.set_mxp_foreground(fg);
                }
                if let Some(bg) = back.and_then(RgbColor::named) {
                    self.output.set_mxp_background(bg);
                }
            }
            Action::High => self.output.set_mxp_flag(TextStyle::Highlight),
            Action::Send => {
                let mxp::SendArgs { href, hint, sendto } = (&args).into();
                let action = href.unwrap_or(SPECIAL_LINK);
                self.output.set_mxp_action(Link::new(action, hint, sendto));
                if action.contains(SPECIAL_LINK) {
                    let template = if sendto == SendTo::Input {
                        format!("echo:{}", action)
                    } else {
                        format!("send:{}", action)
                    };
                    self.mxp_active_tags.set_anchor_template(template);
                }
            }
            Action::Hyperlink => {
                let mxp::HyperlinkArgs { href } = (&args).into();
                let action = href.unwrap_or(SPECIAL_LINK);
                self.output
                    .set_mxp_action(Link::new(action, None, SendTo::Internet));
                if action.contains(SPECIAL_LINK) {
                    self.mxp_active_tags.set_anchor_template(action.to_owned());
                }
            }
            Action::Font => {
                let mxp::FontArgs { fgcolor, bgcolor } = (&args).into();
                for fg in fgcolor {
                    match fg.to_lowercase().as_str() {
                        "blink" => self.output.set_mxp_flag(TextStyle::Blink),
                        "italic" => self.output.set_mxp_flag(TextStyle::Italic),
                        "underline" => self.output.set_mxp_flag(TextStyle::Underline),
                        "bold" => self.output.set_mxp_flag(TextStyle::Bold),
                        "inverse" => self.output.set_mxp_flag(TextStyle::Inverse),
                        color => {
                            if let Some(fg) = RgbColor::named(color) {
                                self.output.set_mxp_foreground(fg);
                            }
                        }
                    };
                }
                if let Some(bg) = bgcolor.and_then(RgbColor::named) {
                    self.output.set_mxp_background(bg);
                }
            }
            Action::Version => self.input.append(
                mxp::responses::identify(&self.config.app_name, &self.config.version).as_bytes(),
            ),
            Action::Afk => {
                let mxp::AfkArgs { challenge } = (&args).into();
                self.output.append_afk(challenge.unwrap_or(""));
            }
            Action::Support => Atom::fmt_supported(self.input.get_mut(), args),
            Action::User => input_mxp_auth(&mut self.input, &self.config.player),
            Action::Password => input_mxp_auth(&mut self.input, &self.config.password),
            Action::Br => {
                self.output.start_line();
            }
            Action::Reset => {
                self.mxp_off(false);
            }
            Action::Mxp => {
                if args.has_keyword(Keyword::Off) {
                    self.mxp_off(true);
                }

                if args.has_keyword(Keyword::DefaultLocked) {
                    self.mxp_mode_default = mxp::Mode::LOCKED;
                } else if args.has_keyword(Keyword::DefaultSecure) {
                    self.mxp_mode_default = mxp::Mode::SECURE;
                } else if args.has_keyword(Keyword::DefaultOpen) {
                    self.mxp_mode_default = mxp::Mode::OPEN;
                }

                if args.has_keyword(Keyword::IgnoreNewlines) {
                    self.output.set_format(TextFormat::Paragraph);
                } else if args.has_keyword(Keyword::UseNewlines) {
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
            Action::Img | Action::Image => {
                if let Some(xch_mode) = args.get("xch_mode") {
                    self.pueblo_active = true;
                    if xch_mode.eq_ignore_ascii_case("purehtml") {
                        self.suppress_newline = true;
                    } else if xch_mode.eq_ignore_ascii_case("html") {
                        self.suppress_newline = false;
                    }
                }
                if let Some(url) = args.get("url").or_else(|| args.get("src")) {
                    let fname = args.get("fname").unwrap_or("");
                    self.output.append_image(format!("{url}{fname}"));
                }
            }
            Action::XchPage => {
                self.pueblo_active = true;
                self.mxp_off(false);
            }
            Action::Var => {
                let variable = args.get(0).unwrap_or("");
                if mxp::is_valid(variable) && mxp::EntityMap::global(variable).is_none() {
                    self.output.set_mxp_variable(variable.to_owned());
                }
            }
            _ => (),
        }
    }

    fn mxp_close_tags_from(&mut self, pos: usize) {
        if let Some(span_index) = self.mxp_active_tags.truncate(pos) {
            self.output.truncate_spans(span_index);
        }
    }

    fn mxp_collected_entity(&mut self) -> Result<(), mxp::ParseError> {
        let mxp_string = self.take_mxp_string()?;
        let name = mxp_string.trim();
        mxp::validate(name, mxp::Error::InvalidEntityName)?;
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
                self.mxp_mode_default = mxp::Mode::OPEN
            }
            mxp::Mode::SECURE_ONCE => self.mxp_mode_previous = self.mxp_mode,
            mxp::Mode::PERM_OPEN | mxp::Mode::PERM_SECURE | mxp::Mode::PERM_LOCKED => {
                self.mxp_mode_default = newmode
            }
            _ => (),
        }
        self.mxp_mode = newmode;
    }

    fn interpret_ansi(&mut self, code: u8) {
        match code {
            ansi::RESET => self.output.reset_ansi(),

            ansi::BOLD => self.output.set_ansi_flag(TextStyle::Bold),
            ansi::BLINK => self.output.set_ansi_flag(TextStyle::Italic),
            ansi::UNDERLINE => self.output.set_ansi_flag(TextStyle::Underline),
            ansi::SLOW_BLINK => self.output.set_ansi_flag(TextStyle::Italic),
            ansi::FAST_BLINK => self.output.set_ansi_flag(TextStyle::Italic),
            ansi::INVERSE => self.output.set_ansi_flag(TextStyle::Inverse),
            ansi::STRIKEOUT => self.output.set_ansi_flag(TextStyle::Strikeout),

            ansi::CANCEL_BOLD => self.output.unset_ansi_flag(TextStyle::Bold),
            ansi::CANCEL_BLINK => self.output.unset_ansi_flag(TextStyle::Italic),
            ansi::CANCEL_UNDERLINE => self.output.unset_ansi_flag(TextStyle::Underline),
            ansi::CANCEL_SLOW_BLINK => self.output.unset_ansi_flag(TextStyle::Italic),
            ansi::CANCEL_FAST_BLINK => self.output.unset_ansi_flag(TextStyle::Italic),
            ansi::CANCEL_INVERSE => self.output.unset_ansi_flag(TextStyle::Inverse),
            ansi::CANCEL_STRIKEOUT => self.output.unset_ansi_flag(TextStyle::Strikeout),

            ansi::FG_256_COLOR => self.phase = Phase::Foreground256Start,
            ansi::BG_256_COLOR => self.phase = Phase::Background256Start,
            ansi::FG_DEFAULT => self.output.set_ansi_foreground(WorldColor::WHITE),
            ansi::BG_DEFAULT => self.output.set_ansi_background(WorldColor::BLACK),
            ansi::FG_BLACK..=ansi::FG_WHITE => self
                .output
                .set_ansi_foreground(WorldColor::Ansi(code - ansi::FG_BLACK)),
            ansi::BG_BLACK..=ansi::BG_WHITE => self
                .output
                .set_ansi_background(WorldColor::Ansi(code - ansi::BG_BLACK)),
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
                    telnet::SGA | telnet::MUD_SPECIFIC => true,
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
                    telnet::CHARSET => true,
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
                    self.handle_mxp_error(mxp::ParseError::new(
                        &self.mxp_string,
                        mxp::Error::UnterminatedElement,
                    ));
                    self.mxp_string.clear();
                }
                b'\'' | b'"' => {
                    self.mxp_string.push(c);
                    self.mxp_quote_terminator = Some(c);
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
                if self.mxp_quote_terminator == Some(c) {
                    self.phase = Phase::MxpElement;
                    self.mxp_quote_terminator = None;
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
                    self.handle_mxp_error(mxp::ParseError::new(
                        &self.mxp_string,
                        mxp::Error::UnterminatedEntity,
                    ));
                    self.mxp_string.clear();
                }
                b'<' => {
                    self.mxp_string.push(c);
                    self.handle_mxp_error(mxp::ParseError::new(
                        &self.mxp_string,
                        mxp::Error::UnterminatedEntity,
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
                    self.mxp_mode = self.mxp_mode_previous
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
