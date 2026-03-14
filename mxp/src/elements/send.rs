use std::borrow::Cow;
use std::str::{self, FromStr};

use crate::keyword::SendKeyword;
use crate::parse::{Decoder, Error, Scan};

/// Specifies that a span of text can be clicked on to cause an action.
///
/// See [MXP specification: `<SEND>`](https://www.zuggsoft.com/zmud/mxp.htm#Links).
///
/// # Examples
///
/// ```
/// assert_eq!(
///     "<SEND HREF='buy bread' HINT='Buy some bread' PROMPT EXPIRE=market>".parse::<mxp::Send>(),
///     Ok(mxp::Send {
///         href: "buy bread".into(),
///         hint: "Buy some bread".into(),
///         expire: Some("market".into()),
///         prompt: true,
///     }),
/// );
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Send<S = String> {
    /// Command to send.
    pub href: S,
    /// Mouseover hint.
    pub hint: S,
    /// Optional scope for the link. If defined, an [`Expire`](crate::Expire) command can
    /// invalidate the link.
    pub expire: Option<S>,
    /// If true, the command should be sent to the client's command line rather than the world.
    pub prompt: bool,
}

impl Send {
    pub const EMBED_ENTITY: &'static str = "&text;";
}

impl<S: From<&'static str>> Default for Send<S> {
    /// Constructs a link with the action `"&text;"`, meaning the surrounded text will be used.
    fn default() -> Self {
        Self {
            href: S::from(Send::EMBED_ENTITY),
            hint: S::from(Send::EMBED_ENTITY),
            expire: None,
            prompt: false,
        }
    }
}

impl<S> Send<S> {
    /// Applies a type transformation to all text, returning a new struct.
    pub fn map_text<T, F>(self, mut f: F) -> Send<T>
    where
        F: FnMut(S) -> T,
    {
        Send {
            href: f(self.href),
            hint: f(self.hint),
            expire: self.expire.map(f),
            prompt: self.prompt,
        }
    }
}

impl_into_owned!(Send);

impl<S: AsRef<str>> Send<S> {
    /// Returns a new struct that borrows text from this one.
    pub fn borrow_text(&self) -> Send<&str> {
        Send {
            href: self.href.as_ref(),
            hint: self.hint.as_ref(),
            expire: self.expire.as_ref().map(AsRef::as_ref),
            prompt: self.prompt,
        }
    }

    /// Returns a copy of the link, replacing `"&text;"` with the supplied text.
    ///
    /// # Examples
    ///
    /// ```
    /// let link_template = mxp::Send::from("do &text;|(&text;)|&text;|other");
    /// let link = link_template.for_text("some text");
    /// assert_eq!(link.href, "do some text|(some text)|some text|other");
    /// ```
    #[must_use = "function returns a new link"]
    pub fn for_text(&self, text: &str) -> Send<String> {
        Send {
            href: self.href.as_ref().replace(Send::EMBED_ENTITY, text),
            hint: self.hint.as_ref().replace(Send::EMBED_ENTITY, text),
            expire: self
                .expire
                .as_ref()
                .map(|expire| expire.as_ref().to_owned()),
            prompt: self.prompt,
        }
    }

    /// If true, the command should be used to create a menu of options.
    pub fn is_menu(&self) -> bool {
        self.href.as_ref().as_bytes().contains(&b'|')
    }
}

impl_partial_eq!(Send);

impl<S: Clone> From<S> for Send<S> {
    fn from(value: S) -> Self {
        Self {
            href: value.clone(),
            hint: value,
            expire: None,
            prompt: false,
        }
    }
}

impl Send<String> {
    /// Iterates through menu options.
    pub fn menu(&self) -> SendMenu<'_> {
        SendMenu {
            commands: self.href.split('|'),
            labels: self.hint.split('|'),
        }
    }
}

impl Send<Cow<'_, str>> {
    /// Iterates through menu options.
    pub fn menu(&self) -> SendMenu<'_> {
        SendMenu {
            commands: self.href.split('|'),
            labels: self.hint.split('|'),
        }
    }
}

impl<'a> Send<&'a str> {
    /// Iterates through menu options.
    pub fn menu(&self) -> SendMenu<'a> {
        let mut labels = self.hint.split('|');
        labels.next();
        SendMenu {
            commands: self.href.split('|'),
            labels,
        }
    }
}

impl<'a, D> TryFrom<Scan<'a, D>> for Send<Cow<'a, str>>
where
    D: Decoder,
{
    type Error = Error;

    fn try_from(scanner: Scan<'a, D>) -> crate::Result<Self> {
        let mut scanner = scanner.with_keywords();
        let href = scanner
            .next_or("href")?
            .unwrap_or(Cow::Borrowed(Send::EMBED_ENTITY));
        let hint = scanner.next_or("hint")?.unwrap_or_else(|| href.clone());
        let expire = scanner.next_or("expire")?;
        let keywords = scanner.into_keywords()?;
        Ok(Self {
            href,
            hint,
            expire,
            prompt: keywords.contains(SendKeyword::Prompt),
        })
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct SendMenuItem<'a> {
    /// Command to send.
    pub command: &'a str,
    /// Label to display.
    pub label: &'a str,
}

#[derive(Clone, Debug)]
pub struct SendMenu<'a> {
    commands: str::Split<'a, char>,
    labels: str::Split<'a, char>,
}

impl<'a> Iterator for SendMenu<'a> {
    type Item = SendMenuItem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let command = self.commands.next()?;
        Some(SendMenuItem {
            command,
            label: self.labels.next().unwrap_or(command),
        })
    }
}

impl FromStr for Send {
    type Err = crate::parse::FromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::parse::parse_element(s, crate::ActionKind::Send)
    }
}
