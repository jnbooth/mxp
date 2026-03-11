use std::str::FromStr;

use super::{Hyperlink, Send};

/// Destination for a [`Link`] element.
///
/// See [`MXP specification: Links`](https://www.zuggsoft.com/zmud/mxp.htm#Links).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum SendTo {
    /// `<SEND href="...">`.
    /// When clicked, the link href should be sent to the server as if typed by the user.
    #[default]
    World = 1,
    /// `<SEND PROMPT href="...">`.
    /// When clicked, the link text should be sent to the client's command line.
    Input,
    /// `<A href="..."`>`.
    /// When clicked, the link text should be opened in a browser as a web URL.
    Internet,
}

/// Prompts displayed in a menu when the user right-clicks on a [`Link`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LinkPrompt {
    /// Action to send.
    pub action: String,
    /// Optional label to display instead of the action text.
    pub label: Option<String>,
}

impl LinkPrompt {
    /// `self.label`, or `self.action` if `self.label` is `None`.
    pub fn label(&self) -> &str {
        self.label.as_deref().unwrap_or(&self.action)
    }

    fn with_text(&self, text: &str) -> Self {
        Self {
            action: embed(&self.action, text),
            label: self.label.clone(),
        }
    }
}

impl From<&str> for LinkPrompt {
    fn from(value: &str) -> Self {
        Self {
            action: value.to_owned(),
            label: None,
        }
    }
}

/// Specifies that a span of text can be clicked on to cause an action, and optionally can be
/// right-clicked on to display a menu of choices.
///
/// This struct encompasses two MXP tags: `<A>` and `<SEND>`. If `send_to` is
/// [`SendTo::Internet`], this value was parsed from an `<A>` tag. Otherwise, it was parsed from a
/// `<SEND>` tag.
///
/// See [`MXP specification: Links`](https://www.zuggsoft.com/zmud/mxp.htm#Links).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Link {
    /// Text to supply to the destination. Depending on the value of `send_to`, this is a URL, a
    /// MUD command, or text to display in the command line.
    ///
    /// If this value is `"&text;"`, the text under the link should be used as the action instead.
    pub action: String,
    /// Flyover hint.
    pub hint: Option<String>,
    /// Right-click prompts for actions.
    pub prompts: Vec<LinkPrompt>,
    /// Where to send the result of clicking on the link.
    pub send_to: SendTo,
    /// Optional scope for the link. If defined, an `<EXPIRE>` command can invalidate the link.
    pub expires: Option<String>,
}

impl Link {
    pub const EMBED_ENTITY: &'static str = "&text;";

    /// Constructs a new `Link`.
    pub fn new(
        action: &str,
        hints: Option<&str>,
        send_to: SendTo,
        expires: Option<String>,
    ) -> Self {
        let (action, actions) = split_list(action);
        match hints {
            None => Self {
                action,
                hint: None,
                prompts: actions,
                send_to,
                expires,
            },
            Some(hints) => {
                let (hint, prompts) = split_list(hints);
                Self {
                    action,
                    hint: Some(hint),
                    prompts: if prompts.is_empty() { actions } else { prompts },
                    send_to,
                    expires,
                }
            }
        }
    }

    /// A simple link that sends the text under it to the server as a command when clicked.
    ///
    /// This is equivalent to `<SEND href="&text;">`.
    pub fn for_text() -> Self {
        Self {
            action: Self::EMBED_ENTITY.to_owned(),
            ..Default::default()
        }
    }

    /// Returns a copy of the link, replacing `"&text;"` with the supplied text.
    ///
    /// # Examples
    ///
    /// ```
    /// let link_template = mxp::Link::from("do &text;|(&text;)|&text;|other");
    /// let link = link_template.with_text("some text");
    /// assert_eq!(link.action, "do some text");
    /// let prompts: Vec<_> = link.prompts.iter().map(|prompt| prompt.action.as_str()).collect();
    /// assert_eq!(prompts, &["(some text)", "some text", "other"]);
    /// ```
    #[must_use = "function returns a new link"]
    pub fn with_text(&self, text: &str) -> Self {
        Self {
            action: embed(&self.action, text),
            hint: self.hint.clone(),
            prompts: self
                .prompts
                .iter()
                .map(|prompt| prompt.with_text(text))
                .collect(),
            send_to: self.send_to,
            expires: self.expires.clone(),
        }
    }
}

impl<'a> From<&'a str> for Link {
    /// Equivalent to `Self::new(action, None, mxp::SendTo::World, None)`.
    fn from(action: &'a str) -> Self {
        Self::new(action, None, SendTo::World, None)
    }
}

fn embed(action: &str, text: &str) -> String {
    action.replace(Link::EMBED_ENTITY, text)
}

fn split_list(list: &str) -> (String, Vec<LinkPrompt>) {
    let mut iter = list.split('|');
    let first = iter.next().unwrap().to_owned();
    (first, iter.map(LinkPrompt::from).collect())
}

impl<S> From<Hyperlink<S>> for Link
where
    S: AsRef<str>,
{
    fn from(value: Hyperlink<S>) -> Self {
        Self::new(
            value.href.as_ref(),
            value.hint.as_ref().map(AsRef::as_ref),
            SendTo::Internet,
            value.expire.map(|expire| expire.as_ref().to_owned()),
        )
    }
}

impl<S> From<Send<S>> for Link
where
    S: AsRef<str>,
{
    fn from(value: Send<S>) -> Self {
        Self::new(
            value
                .href
                .as_ref()
                .map_or(Link::EMBED_ENTITY, AsRef::as_ref),
            value.hint.as_ref().map(AsRef::as_ref),
            value.send_to,
            value.expire.map(|expire| expire.as_ref().to_owned()),
        )
    }
}

impl FromStr for Link {
    type Err = crate::parse::FromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let source = crate::parse::cleanup_source(s)?;
        let mut words = crate::parse::Words::new(source);
        let name = words.validate_next_or(crate::ErrorKind::InvalidElementName)?;
        let Some(tag) = crate::element::Tag::well_known(name) else {
            return Err(Self::Err::UnexpectedTag(name.to_owned()));
        };
        let args = words.parse_args()?;
        let scanner = args.scan(());
        match tag.action {
            crate::ActionKind::Hyperlink => Ok(Hyperlink::try_from(scanner)?.into()),
            crate::ActionKind::Send => Ok(Send::try_from(scanner)?.into()),
            _ => Err(Self::Err::UnexpectedTag(name.to_owned())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(strs: &[&str]) -> Vec<LinkPrompt> {
        strs.iter().map(|&s| LinkPrompt::from(s)).collect()
    }

    #[test]
    fn multi_action_link() {
        let link = Link::new("a|b|c|d", Some("e"), SendTo::World, None);
        assert_eq!(
            (link.action.as_str(), link.prompts),
            ("a", strings(&["b", "c", "d"]))
        );
    }

    #[test]
    fn multi_hint_link() {
        let link = Link::new("a|b|c|d", Some("e|f|g"), SendTo::World, None);
        assert_eq!(
            (link.hint, link.prompts),
            (Some("e".to_owned()), strings(&["f", "g"]))
        );
    }
}
