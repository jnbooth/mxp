use std::borrow::Cow;
use std::{fmt, str};

use crate::arguments::{ArgumentScanner, FromArgs};
use crate::keyword::SendKeyword;

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
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
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

    fn embed_text(input: &str, text: &str) -> String {
        if input.is_empty() {
            text.to_owned()
        } else {
            input.replace(Send::EMBED_ENTITY, text)
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
        let href_str = self.href.as_ref();
        let hint_str = self.hint.as_ref();
        let href = Send::embed_text(href_str, text);
        let hint = if hint_str == href_str {
            href.clone()
        } else {
            Send::embed_text(hint_str, text)
        };
        Send {
            href,
            hint,
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
    /// Iterator that produces menu items for a `Send`, as parsed from the [`href`](Self::href) and
    /// [`hint`](Self::hint) fields.
    ///
    /// See [MXP specification: `<SEND>`](https://www.zuggsoft.com/zmud/mxp.htm#Links).
    pub fn menu(&self) -> SendMenu<'_> {
        SendMenu {
            commands: self.href.split('|'),
            labels: self.hint.split('|'),
        }
    }
}

impl Send<Cow<'_, str>> {
    /// Iterator that produces menu items for a `Send`, as parsed from the [`href`](Self::href) and
    /// [`hint`](Self::hint) fields.
    ///
    /// See [MXP specification: `<SEND>`](https://www.zuggsoft.com/zmud/mxp.htm#Links).
    pub fn menu(&self) -> SendMenu<'_> {
        SendMenu {
            commands: self.href.split('|'),
            labels: self.hint.split('|'),
        }
    }
}

impl<'a> Send<&'a str> {
    /// Iterator that produces menu items for a `Send`, as parsed from the [`href`](Self::href) and
    /// [`hint`](Self::hint) fields.
    ///
    /// See [MXP specification: `<SEND>`](https://www.zuggsoft.com/zmud/mxp.htm#Links).
    pub fn menu(&self) -> SendMenu<'a> {
        let mut labels = self.hint.split('|');
        labels.next();
        SendMenu {
            commands: self.href.split('|'),
            labels,
        }
    }
}

impl<'a, S: AsRef<str> + Clone + Default> FromArgs<'a, S> for Send<S> {
    fn from_args<A: ArgumentScanner<'a, Decoded = S>>(scanner: A) -> crate::Result<Self> {
        let mut scanner = scanner.with_keywords();
        let href = scanner.get_next_or("href")?.unwrap_or_default();
        let hint = scanner.get_next_or("hint")?.unwrap_or_else(|| href.clone());
        let expire = scanner.get_next_or("expire")?;
        let keywords = scanner.into_keywords()?;
        Ok(Self {
            href,
            hint,
            expire,
            prompt: keywords.contains(SendKeyword::Prompt),
        })
    }
}

impl_from_str!(Send);

/// This struct is created by [`Send::menu`]. See its documentation for more.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct SendMenuItem<'a> {
    /// Command to send.
    pub command: &'a str,
    /// Label to display.
    pub label: &'a str,
}

/// This struct is created by [`Send::menu`]. See its documentation for more.
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Clone)]
pub struct SendMenu<'a> {
    commands: str::Split<'a, char>,
    labels: str::Split<'a, char>,
}

impl fmt::Debug for SendMenu<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
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

impl<S: AsRef<str>> fmt::Display for Send<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Send {
            mut href,
            mut hint,
            expire,
            prompt,
        } = self.borrow_text();
        if hint == href {
            hint = "";
        }
        if href == Send::EMBED_ENTITY {
            href = "";
        }
        crate::display::ElementFormatter {
            name: "SEND",
            arguments: &[&href, &hint, &expire],
            keywords: &[("PROMPT", prompt)],
        }
        .fmt(f)
    }
}
