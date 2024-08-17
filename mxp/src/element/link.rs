use enumeration::Enum;

use crate::argument::{Decoder, ExpectArg, Scan};
use crate::keyword::SendKeyword;
use crate::parser::Error;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Enum)]
pub enum SendTo {
    World,
    Input,
    Internet,
}

impl Default for SendTo {
    fn default() -> Self {
        Self::World
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Link {
    pub action: String,
    /// Flyover hint.
    pub hint: Option<String>,
    /// Right-click prompts for actions.
    pub prompts: Vec<String>,
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

    #[must_use = "function returns a new link"]
    pub fn with_text(&self, text: &str) -> Self {
        Self {
            action: embed(&self.action, text),
            hint: self.hint.clone(),
            prompts: self
                .prompts
                .iter()
                .map(|prompt| embed(prompt, text))
                .collect(),
            sendto: self.sendto,
            expires: self.expires.clone(),
        }
    }
}

fn embed(action: &str, text: &str) -> String {
    action.replace(Link::EMBED_ENTITY, text)
}

fn split_list(list: &str) -> (String, Vec<String>) {
    let mut iter = list.split('|');
    let first = iter.next().unwrap().to_owned();
    (first, iter.map(ToOwned::to_owned).collect())
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HyperlinkArgs<S> {
    pub href: S,
    pub hint: Option<S>,
    pub expire: Option<S>,
}

impl<'a, D: Decoder> TryFrom<Scan<'a, D>> for HyperlinkArgs<D::Output<'a>> {
    type Error = Error;

    fn try_from(mut scanner: Scan<'a, D>) -> crate::Result<Self> {
        Ok(Self {
            href: scanner.next_or("href")?.expect_some("href")?,
            hint: scanner.next_or("hint")?,
            expire: scanner.next_or("expire")?,
        })
    }
}

impl<S: AsRef<str>> From<HyperlinkArgs<S>> for Link {
    fn from(value: HyperlinkArgs<S>) -> Self {
        Self::new(
            value.href.as_ref(),
            value.hint.as_ref().map(AsRef::as_ref),
            SendTo::Internet,
            value.expire.map(|expire| expire.as_ref().to_owned()),
        )
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SendArgs<S> {
    pub href: Option<S>,
    pub hint: Option<S>,
    pub sendto: SendTo,
    pub expire: Option<S>,
}

impl<'a, D: Decoder> TryFrom<Scan<'a, D>> for SendArgs<D::Output<'a>> {
    type Error = Error;

    fn try_from(scanner: Scan<'a, D>) -> crate::Result<Self> {
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

impl<S: AsRef<str>> From<SendArgs<S>> for Link {
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

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExpireArgs<S> {
    pub name: Option<S>,
}

impl<'a, D: Decoder> TryFrom<Scan<'a, D>> for ExpireArgs<D::Output<'a>> {
    type Error = Error;

    fn try_from(mut scanner: Scan<'a, D>) -> crate::Result<Self> {
        Ok(Self {
            name: scanner.next()?,
        })
    }
}
