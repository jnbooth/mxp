use std::borrow::Cow;

use crate::argument::{Decoder, ExpectArg, Scan};
use crate::keyword::SendKeyword;
use crate::parser::Error;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum SendTo {
    #[default]
    World,
    Input,
    Internet,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LinkPrompt {
    /// Action to send.
    pub action: String,
    /// Optional label to display instead of the action text.
    pub label: Option<String>,
}

impl LinkPrompt {
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

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Link {
    pub action: String,
    /// Flyover hint.
    pub hint: Option<String>,
    /// Right-click prompts for actions.
    pub prompts: Vec<LinkPrompt>,
    /// Where to send the result of clicking on the link.
    pub sendto: SendTo,
    /// Optional scope for the link.
    pub expires: Option<String>,
}

impl Link {
    pub const EMBED_ENTITY: &'static str = "&text;";

    pub fn new(action: &str, hints: Option<&str>, sendto: SendTo, expires: Option<String>) -> Self {
        let (action, actions) = split_list(action);
        match hints {
            None => Self {
                action,
                hint: None,
                prompts: actions,
                sendto,
                expires,
            },
            Some(hints) => {
                let (hint, prompts) = split_list(hints);
                Self {
                    action,
                    hint: Some(hint),
                    prompts: if prompts.is_empty() { actions } else { prompts },
                    sendto,
                    expires,
                }
            }
        }
    }

    pub fn for_text() -> Self {
        Self {
            action: Self::EMBED_ENTITY.to_owned(),
            ..Default::default()
        }
    }

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
            sendto: self.sendto,
            expires: self.expires.clone(),
        }
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

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct HyperlinkArgs<S> {
    pub href: S,
    pub hint: Option<S>,
    pub expire: Option<S>,
}

impl<'a, D, S> TryFrom<Scan<'a, D, S>> for HyperlinkArgs<Cow<'a, str>>
where
    D: Decoder,
    S: AsRef<str>,
{
    type Error = Error;

    fn try_from(mut scanner: Scan<'a, D, S>) -> crate::Result<Self> {
        Ok(Self {
            href: scanner.next_or("href")?.expect_some("href")?,
            hint: scanner.next_or("hint")?,
            expire: scanner.next_or("expire")?,
        })
    }
}

impl<S> From<HyperlinkArgs<S>> for Link
where
    S: AsRef<str>,
{
    fn from(value: HyperlinkArgs<S>) -> Self {
        Self::new(
            value.href.as_ref(),
            value.hint.as_ref().map(AsRef::as_ref),
            SendTo::Internet,
            value.expire.map(|expire| expire.as_ref().to_owned()),
        )
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct SendArgs<S> {
    pub href: Option<S>,
    pub hint: Option<S>,
    pub sendto: SendTo,
    pub expire: Option<S>,
}

impl<'a, D, S> TryFrom<Scan<'a, D, S>> for SendArgs<Cow<'a, str>>
where
    D: Decoder,
    S: AsRef<str>,
{
    type Error = Error;

    fn try_from(scanner: Scan<'a, D, S>) -> crate::Result<Self> {
        let mut scanner = scanner.with_keywords();
        Ok(Self {
            href: scanner.next_or("href")?,
            hint: scanner.next_or("hint")?,
            expire: scanner.next_or("expire")?,
            sendto: if scanner.into_keywords().contains(SendKeyword::Prompt) {
                SendTo::Input
            } else {
                SendTo::World
            },
        })
    }
}

impl<S> From<SendArgs<S>> for Link
where
    S: AsRef<str>,
{
    fn from(value: SendArgs<S>) -> Self {
        Self::new(
            value
                .href
                .as_ref()
                .map_or(Link::EMBED_ENTITY, AsRef::as_ref),
            value.hint.as_ref().map(AsRef::as_ref),
            value.sendto,
            value.expire.map(|expire| expire.as_ref().to_owned()),
        )
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ExpireArgs<S> {
    pub name: Option<S>,
}

impl<'a, D, S> TryFrom<Scan<'a, D, S>> for ExpireArgs<Cow<'a, str>>
where
    D: Decoder,
    S: AsRef<str>,
{
    type Error = Error;

    fn try_from(mut scanner: Scan<'a, D, S>) -> crate::Result<Self> {
        Ok(Self {
            name: scanner.next()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(strs: &[&str]) -> Vec<LinkPrompt> {
        strs.iter().map(|&s| LinkPrompt::from(s)).collect()
    }

    #[test]
    fn link_embedding() {
        let link = Link::new("do &text;|(&text;)|&text;|other", None, SendTo::World, None)
            .with_text("input");
        assert_eq!(
            (link.action.as_str(), link.prompts),
            ("do input", strings(&["(input)", "input", "other"]))
        );
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
