use std::{mem, vec};

use enumeration::Enum;

use super::config::{AutoConnect, TransformerConfig, UseMxp};
use super::input::{self, BufferedInput};
use super::phase::Phase;
use crate::color::{HexColor, WorldColor};
use crate::escape::{ansi, telnet, utf8};
use crate::mxp;
use crate::style::{BufferedOutput, Heading, InList, OutputFragment, TextFormat, TextStyle};

fn input_mxp_auth(input: &mut BufferedInput, auth: &str, connect: Option<AutoConnect>) {
    if connect != Some(AutoConnect::Mxp) || auth.is_empty() {
        return;
    }
    input.append(auth.as_bytes());
    input.append(b"\r\n");
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SideEffect {
    EnableCompression,
    DisableCompression,
    Beep,
    EraseLine,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Enum)]
pub enum Mccp {
    V1,
    V2,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Enum)]
pub enum ListMode {
    Ordered,
    Unordered,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Transformer {
    config: TransformerConfig,

    phase: Phase,

    mxp_active: bool,
    pueblo_active: bool,
    compressing: bool,
    mccp_ver: Option<Mccp>,
    supports_mccp_2: bool,
    no_echo: bool,

    mxp_script: bool,
    suppress_newline: bool,
    mxp_mode_default: mxp::Mode,
    mxp_mode: mxp::Mode,
    mxp_mode_previous: mxp::Mode,
    mxp_quote_terminator: Option<u8>,
    mxp_string: Vec<u8>,
    mxp_active_tags: Vec<mxp::Tag>,
    mxp_elements: mxp::ElementMap,
    mxp_entities: mxp::EntityMap,
    last_outstanding_tag_count: u64,
    list_mode: Option<ListMode>,
    list_index: u16,

    linecount: u64,
    last_line_with_iac_ga: u64,
    subnegotiation_type: u8,
    subnegotiation_data: Vec<u8>,
    ttype_sequence: u8,
    naws_wanted: bool,
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

            mxp_active: false,
            pueblo_active: false,
            compressing: false,
            mccp_ver: None,
            supports_mccp_2: false,
            no_echo: false,

            mxp_script: false,
            suppress_newline: false,
            mxp_mode_default: mxp::Mode::OPEN,
            mxp_mode: mxp::Mode::OPEN,
            mxp_mode_previous: mxp::Mode::OPEN,
            mxp_quote_terminator: None,
            mxp_string: Vec::new(),
            mxp_active_tags: Vec::new(),
            mxp_elements: mxp::ElementMap::new(),
            mxp_entities: mxp::EntityMap::new(),
            last_outstanding_tag_count: 0,
            list_mode: None,
            list_index: 0,

            linecount: 0,
            last_line_with_iac_ga: 0,
            subnegotiation_type: 0,
            subnegotiation_data: Vec::new(),
            ttype_sequence: 0,
            naws_wanted: false,
            ansi_code: 0,
            ansi_red: 0,
            ansi_green: 0,
            ansi_blue: 0,

            utf8_sequence: Vec::with_capacity(4),
            output,
            input: BufferedInput::new(config.send_mxp_afk_response),
            config,
        }
    }

    pub fn set_config(&mut self, config: TransformerConfig) {
        if config.ignore_mxp_colors {
            self.output.disable_mxp_colors();
        } else {
            self.output.enable_mxp_colors();
        }
        self.input.set_remember(config.send_mxp_afk_response);
        self.config = config;
    }

    pub fn drain_output(&mut self) -> vec::Drain<OutputFragment> {
        self.output.drain()
    }

    pub fn drain_input(&mut self) -> input::Drain {
        self.input.drain()
    }

    pub(super) fn handle_mxp_error(&self, err: mxp::ParseError) {
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

    pub(super) fn mxp_off(&mut self, completely: bool) {
        self.output.reset();

        if !self.mxp_active {
            return;
        }

        let closed = match self.mxp_active_tags.iter().rposition(|x| x.no_reset) {
            None => 0,
            Some(i) => i + 1,
        };
        self.mxp_close_tags_from(closed);
        self.mxp_script = false;
        self.list_mode = None;
        self.list_index = 0;

        if !completely {
            return;
        }
        self.mxp_mode_change(Some(mxp::Mode::OPEN));
        if self.phase.is_mxp() {
            self.phase = Phase::Normal;
        }
        self.pueblo_active = false;
        self.mxp_active = false;

        // self.plugins.send_to_all(Callback::MxpStop, ());
    }

    pub(super) fn mxp_on(&mut self, pueblo: bool, manual: bool) {
        if self.mxp_active {
            return;
        }

        // self.plugins.send_to_all(Callback::MxpStart, ());

        self.mxp_active = true;
        self.pueblo_active = pueblo;
        self.mxp_script = false;
        self.last_outstanding_tag_count = 0;
        self.list_mode = None;

        if manual {
            return;
        }

        self.mxp_mode_default = mxp::Mode::OPEN;
        self.mxp_mode = mxp::Mode::OPEN;
        self.mxp_active_tags.clear();
        self.mxp_elements.clear();
    }

    fn mxp_findtag(&self, secure: bool, name: &str) -> Result<(usize, &mxp::Tag), mxp::ParseError> {
        for (i, tag) in self.mxp_active_tags.iter().enumerate().rev() {
            if tag.name.eq_ignore_ascii_case(name) {
                if !secure && tag.secure {
                    return Err(mxp::ParseError::new(
                        name,
                        mxp::Error::TagOpenedInSecureMode,
                    ));
                } else {
                    return Ok((i, tag));
                }
            }
            if !secure && tag.secure {
                return Err(mxp::ParseError::new(
                    tag,
                    mxp::Error::OpenTagBlockedBySecureTag,
                ));
            }
        }
        Err(mxp::ParseError::new(name, mxp::Error::OpenTagNotThere))
    }

    fn mxp_endtag(&mut self, tag_body: &str) -> Result<(), mxp::ParseError> {
        let was_secure = self.mxp_mode.is_secure();
        self.mxp_restore_mode();
        let mut words = mxp::Words::new(tag_body);
        let name = words.validate_next_or(mxp::Error::InvalidElementName)?;

        if words.next().is_some() {
            return Err(mxp::ParseError::new(
                tag_body,
                mxp::Error::ArgumentsToClosingTag,
            ));
        }

        let (closed, _tag) = self.mxp_findtag(was_secure, name)?;
        /*
        if let Some(template) = &tag.anchor_template {
            let select = self.cursor.document().select(tag.text_index..);
            let fmt = QTextCharFormat::new();
            let text = select.text();
            let anchor = template.replace("&text;", &text);
            fmt.set_anchor_href(&anchor);
            select.merge_char_format(&fmt);
        }
            */
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
        let mut words = mxp::Words::new(tag);

        let definition = words.validate_next_or(mxp::Error::InvalidDefinition)?;
        let name = words.validate_next_or(mxp::Error::InvalidElementName)?;
        match definition.to_lowercase().as_str() {
            "element" | "el" => self.mxp_make_element(name, words),
            "entity" | "en" => self.mxp_make_entity(name, words),
            "attlist" | "at" => self.mxp_make_attributes(name, words),
            _ => Err(mxp::ParseError::new(
                definition,
                mxp::Error::InvalidDefinition,
            )),
        }
    }

    fn mxp_make_element(&mut self, name: &str, words: mxp::Words) -> Result<(), mxp::ParseError> {
        let args = mxp::Arguments::parse_words(words)?;
        if args.has_keyword(mxp::Keyword::Delete) {
            self.mxp_elements.remove(&name);
            return Ok(());
        }
        let el = mxp::Element::parse(name.to_owned(), args)?;
        self.mxp_elements.insert(name.to_owned(), el);
        Ok(())
    }

    fn mxp_make_entity(&mut self, key: &str, mut words: mxp::Words) -> Result<(), mxp::ParseError> {
        if mxp::EntityMap::global(key).is_some() {
            return Err(mxp::ParseError::new(key, mxp::Error::CannotRedefineEntity));
        }
        match words.next() {
            Some(body) // once told me
                if !words.any(|word| {
                    word.eq_ignore_ascii_case("delete") || word.eq_ignore_ascii_case("remove")
                }) =>
            {
                let value = self.mxp_entities.decode(body)?;
                // self.plugins.send_to_all(Callback::MxpSetEntity, format!("{}={}", key, value));
                self.mxp_entities.insert(key.to_owned(), value)
            }
            _ => self.mxp_entities.remove(key),
        };
        Ok(())
    }

    fn mxp_make_attributes(&mut self, key: &str, words: mxp::Words) -> Result<(), mxp::ParseError> {
        self.mxp_elements
            .get_mut(key)
            .ok_or_else(|| mxp::ParseError::new(key, mxp::Error::UnknownElementInAttlist))?
            .attributes
            .append(words)
    }

    pub(super) fn mxp_collected_element(&mut self) -> Result<(), mxp::ParseError> {
        let tag = *self
            .mxp_string
            .first()
            .ok_or_else(|| mxp::ParseError::new("collected element", mxp::Error::EmptyElement))?;
        let text = self.take_mxp_string()?;

        match tag {
            b'!' => self.mxp_definition(&text[1..]),
            b'/' => self.mxp_endtag(&text[1..]),
            _ => self.mxp_start_tag(&text),
        }
    }

    fn mxp_start_tag(&mut self, tag: &str) -> Result<(), mxp::ParseError> {
        let secure = self.mxp_mode.is_secure();
        self.mxp_restore_mode();
        let mut words = mxp::Words::new(tag);
        let name = words.validate_next_or(mxp::Error::InvalidElementName)?;
        let component = self.mxp_elements.get_component(name)?;
        let flags = component.flags();
        self.mxp_active_tags.push(mxp::Tag {
            name: name.to_owned(),
            secure,
            no_reset: flags.contains(mxp::TagFlag::NoReset),
            span_index: self.output.span_len(),
            anchor_template: None,
        });
        if !flags.contains(mxp::TagFlag::Open) && !secure {
            return Err(mxp::ParseError::new(name, mxp::Error::ElementWhenNotSecure));
        }
        // let argstring = words.as_str();
        let mut args = mxp::Arguments::parse_words(words)?;

        /*
        if name != "afk"
            && self.plugins.send_to_all_until(
                Callback::MxpOpenTag,
                (format!("{},{}", name, argstring), &args),
                enums![true],
            )
        {
            return Ok(());
        }
        */

        if let Some(variable) = component.variable() {
            self.output.set_mxp_variable(variable);
        }

        match component {
            mxp::ElementComponent::Atom(atom) => {
                for value in args.values_mut() {
                    *value = self.mxp_entities.decode(value)?;
                }
                self.mxp_open_atom(atom.action, args);
            }
            mxp::ElementComponent::Custom(el) => {
                // create a temporary vector to avoid borrow conflict
                // could clone the element instead, but that seems like a waste
                let actions: Result<Vec<_>, mxp::ParseError> = el
                    .items
                    .iter()
                    .map(|item| {
                        let mut newargs = mxp::Arguments::new();
                        for (i, arg) in &item.arguments {
                            let val = self.mxp_entities.decode_el(el, arg, &args)?;
                            match i {
                                mxp::ArgumentIndex::Positional(..) => newargs.push(val),
                                mxp::ArgumentIndex::Named(key) => newargs.set(key, val),
                            }
                        }
                        Ok((item.atom.action, newargs))
                    })
                    .collect();
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
                let mut scanner = args.scan();
                if let Some(fg) = scanner.next_or(&["fore"]).and_then(HexColor::named) {
                    self.output.set_mxp_foreground(fg);
                }
                if let Some(bg) = scanner.next_or(&["back"]).and_then(HexColor::named) {
                    self.output.set_mxp_background(bg);
                }
            }
            Action::High => {
                self.output.set_mxp_flag(TextStyle::Highlight);
            }
            Action::Send => {
                let mut scanner = args.scan();
                let action = scanner
                    .next_or(&["href", "xch_cmd"])
                    .unwrap_or(SPECIAL_LINK);
                let hint = scanner.next_or(&["hint", "xch_hint"]);
                let sendto = if args.has_keyword(Keyword::Prompt) {
                    SendTo::Input
                } else {
                    SendTo::World
                };
                self.output.set_mxp_action(Link::new(action, hint, sendto));
                if action.contains(SPECIAL_LINK) {
                    if let Some(tag) = self.mxp_active_tags.last_mut() {
                        let template = if args.has_keyword(Keyword::Prompt) {
                            format!("echo:{}", action)
                        } else {
                            format!("send:{}", action)
                        };
                        tag.anchor_template = Some(template);
                    }
                }
            }
            Action::Hyperlink => {
                let mut scanner = args.scan();
                let action = scanner.next_or(&["href"]).unwrap_or(SPECIAL_LINK);
                self.output
                    .set_mxp_action(Link::new(action, None, SendTo::Internet));
                if action.contains(SPECIAL_LINK) {
                    if let Some(tag) = self.mxp_active_tags.last_mut() {
                        tag.anchor_template = Some(action.to_owned());
                    }
                }
            }
            Action::Font => {
                let mut scanner = args.scan();
                for fg in scanner
                    .next_or(&["color", "fgcolor"])
                    .unwrap_or("")
                    .split(',')
                {
                    match fg.to_lowercase().as_str() {
                        "blink" => self.output.set_mxp_flag(TextStyle::Blink),
                        "italic" => self.output.set_mxp_flag(TextStyle::Italic),
                        "underline" => self.output.set_mxp_flag(TextStyle::Underline),
                        "bold" => self.output.set_mxp_flag(TextStyle::Bold),
                        "inverse" => self.output.set_mxp_flag(TextStyle::Inverse),
                        color => {
                            if let Some(fg) = HexColor::named(color) {
                                self.output.set_mxp_foreground(fg);
                            }
                        }
                    };
                }
                if let Some(bg) = scanner
                    .next_or(&["back", "bgcolor"])
                    .and_then(HexColor::named)
                {
                    self.output.set_mxp_background(bg);
                }
            }
            Action::Version => {
                self.input.append(b"\x1B[1zVERSION MXP=\"");
                self.input.append(mxp::VERSION.as_bytes());
                self.input.append(b"\" CLIENT=");
                self.input.append(self.config.app_name.as_bytes());
                self.input.append(b" VERSION=\"");
                self.input.append(self.config.version.as_bytes());
                self.input.append(b"\" REGISTERED=YES>\n");
            }
            Action::Afk => {
                let mut scanner = args.scan();
                if let Some(afk) = self.input.afk() {
                    let challenge = scanner.next_or(&["challenge"]).unwrap_or("");
                    self.input.append(b"\x1B[1z<AFK ");
                    self.input.append(afk.as_secs().to_string().as_bytes());
                    self.input.append(challenge.as_bytes());
                    self.input.append(b">\n");
                }
            }
            Action::Support => {
                Atom::fmt_supported(self.input.get_mut(), args);
            }
            Action::User => input_mxp_auth(
                &mut self.input,
                &self.config.player,
                self.config.connect_method,
            ),
            Action::Password => input_mxp_auth(
                &mut self.input,
                &self.config.password,
                self.config.connect_method,
            ),
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
            Action::Script => {
                self.mxp_script = true;
            }
            Action::Hr => {
                self.output.append_hr();
            }
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
                    self.output.append(&i.to_string());
                    self.output.append(b". ");
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
                    // TODO setting on MXP page to enable or disable images
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
        let tag = match self.mxp_active_tags.get(pos) {
            Some(tag) => tag,
            None => return,
        };
        self.output.truncate_spans(tag.span_index);
        self.mxp_active_tags.truncate(pos);
    }

    pub(super) fn mxp_collected_entity(&mut self) -> Result<(), mxp::ParseError> {
        let mxp_string = self.take_mxp_string()?;
        let name = mxp_string.trim();
        mxp::validate(name, mxp::Error::InvalidEntityName)?;
        if let Some(entity) = self.mxp_entities.get(name)? {
            self.mxp_active = false;
            self.output.append(entity);
            self.mxp_active = true;
        }
        Ok(())
    }

    pub(super) fn mxp_mode_change(&mut self, newmode: Option<mxp::Mode>) {
        let oldmode = self.mxp_mode;
        let newmode = newmode.unwrap_or(self.mxp_mode_default);
        let closing = oldmode.is_open() && !newmode.is_open();
        if closing {
            let closed = match self.mxp_active_tags.iter().rposition(|x| x.secure) {
                None => 0,
                Some(i) => i + 1,
            };
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
            _ => {
                if let Some(fg) = WorldColor::fg_from_ansi(code) {
                    self.output.set_ansi_foreground(fg);
                } else if let Some(bg) = WorldColor::bg_from_ansi(code) {
                    self.output.set_ansi_background(bg);
                }
            }
        }
    }

    fn build_ansi_color(&self) -> HexColor {
        HexColor::rgb(self.ansi_red, self.ansi_green, self.ansi_blue)
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
                self.output.set_ansi_foreground(self.ansi_code);
                self.phase = Phase::Normal;
            }
            Phase::Background256Finish => {
                self.output.set_ansi_background(self.ansi_code);
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

    pub fn interpret_char(&mut self, c: u8) -> Option<SideEffect> {
        let last_char = self.output.last().unwrap_or(b'\n');

        if last_char == b'\r' && c != b'\n' {
            return Some(SideEffect::EraseLine);
        }

        if self.phase == Phase::Utf8Character && !utf8::is_continuation(c) {
            self.output.append(&mut self.utf8_sequence);
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
                        self.last_line_with_iac_ga = self.linecount;
                        // self.plugins.send_to_all(Callback::IacGa, ());
                        if self.config.convert_ga_to_newline {
                            self.output.append(b'\n');
                        }
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
                    // _ => self.tellnet_callbacks(c, "WILL", "SENT_DO"),
                    _ => false,
                };
                let verb = if will { telnet::DO } else { telnet::DONT };
                self.input.append(&[telnet::IAC, verb, c]);
            }

            Phase::Wont => {
                self.phase = Phase::Normal;
                if !self.config.no_echo_off {
                    self.no_echo = false;
                }
                self.input.append(&[telnet::IAC, telnet::DONT, c]);
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
                        self.naws_wanted = true;
                        // self.send_window_sizes(self.world.wrap_column)?;
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
                    // _ => self.telnet_callbacks(c, "DO", "SENT_WILL"),
                    _ => false,
                };
                let verb = if will { telnet::DO } else { telnet::DONT };
                self.input.append(&[telnet::IAC, verb, c]);
            }

            Phase::Dont => {
                self.phase = Phase::Normal;
                match c {
                    telnet::MXP if self.mxp_active => self.mxp_off(true),
                    telnet::TERMINAL_TYPE => self.ttype_sequence = 0,
                    _ => (),
                }
                self.input.append(&[telnet::IAC, telnet::WONT, c]);
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

            Phase::CompressWill if c == telnet::SE => {
                self.mccp_ver = Some(Mccp::V1);
                return Some(SideEffect::EnableCompression);
            }
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
                            self.mccp_ver = Some(Mccp::V2);
                            return Some(SideEffect::EnableCompression);
                        }
                    }
                    telnet::MXP => {
                        if self.config.use_mxp == UseMxp::Command {
                            self.mxp_on(false, false);
                        }
                    }
                    telnet::TERMINAL_TYPE => {
                        if self.subnegotiation_data.first() == Some(&telnet::TTYPE_SEND) {
                            match self.ttype_sequence {
                                0 => {
                                    self.ttype_sequence += 1;
                                    self.input.append(&[
                                        telnet::IAC,
                                        telnet::SB,
                                        telnet::TERMINAL_TYPE,
                                        telnet::TTYPE_IS,
                                    ]);
                                    let ttype = self.config.terminal_identification.as_bytes();
                                    let trimmed = if ttype.len() > 20 {
                                        &ttype[..20]
                                    } else {
                                        ttype
                                    };
                                    self.input.append(trimmed);
                                    self.input.append(&[telnet::IAC, telnet::SE]);
                                }
                                1 => {
                                    self.ttype_sequence += 1;
                                    self.input.append(telnet::TTYPE_ANSI)
                                }
                                _ if self.config.utf_8 => self.input.append(telnet::TTYPE_UTF8),
                                _ => self.input.append(telnet::TTYPE_XTERM),
                            }
                        }
                    }
                    telnet::CHARSET => {
                        let data = &self.subnegotiation_data;
                        if data.len() >= 3 && data[0] == 1 {
                            self.input
                                .append(telnet::find_charset(data, self.config.utf_8));
                        }
                    }
                    telnet::MUD_SPECIFIC => {
                        /*
                        let data = String::from_utf8_lossy(&self.subnegotiation_data);
                        // self.plugins.send_to_all(Callback::TelnetOption, data);
                        */
                    }
                    _ => {
                        /*
                        let sbtype = self.subnegotiation_type;
                        let data = String::from_utf8_lossy(&self.subnegotiation_data);
                        self.plugins
                            .send_to_all(Callback::TelnetSubnegotiation, (sbtype, data));
                        */
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
                telnet::IAC => {
                    if self.phase == Phase::Iac {
                        self.output.append(c);
                        self.phase = Phase::Normal;
                    } else {
                        self.phase = Phase::Iac;
                    }
                }
                b'<' if self.mxp_active && self.mxp_mode.is_mxp() => {
                    self.mxp_string.clear();
                    self.phase = Phase::MxpElement;
                }
                b'&' if self.mxp_active && self.mxp_mode.is_mxp() => {
                    self.mxp_string.clear();
                    self.phase = Phase::MxpEntity;
                }
                b'\x07' => return Some(SideEffect::Beep),
                b'\t' if self.output.format().contains(TextFormat::Paragraph) => {
                    if last_char != b' ' {
                        self.output.append(b' ');
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
                            b'.' => self.output.append(b"  "),
                            b' ' | b'\t' | b'\x0C' => (),
                            _ => self.output.append(b' '),
                        }
                    } else if !self.suppress_newline && !format.contains(TextFormat::Pre) {
                        self.output.start_line();
                    }
                }
                _ if utf8::is_higher_order(c) => {
                    self.utf8_sequence.push(c);
                    self.phase = Phase::Utf8Character;
                }
                _ => self.output.append(c),
            },
        }

        None
    }
}
