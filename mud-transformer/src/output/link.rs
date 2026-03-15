/// Destination for a [`Link`] element.
///
/// See [`MXP specification: Links`](https://www.zuggsoft.com/zmud/mxp.htm#Links).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum SendTo {
    /// `<SEND href="...">`.
    /// When clicked, the link href should be sent to the server as if typed by the user.
    #[default]
    World,
    /// `<SEND PROMPT href="...">`.
    /// When clicked, the link text should be sent to the client's command line.
    Prompt,
    /// `<A href="..."`>`.
    /// When clicked, the link text should be opened in a browser as a web URL.
    Internet,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Link {
    pub href: String,
    pub hint: String,
    pub expire: Option<String>,
    pub send_to: SendTo,
    pub menu: bool,
}

impl Default for Link {
    fn default() -> Self {
        Self {
            href: mxp::Send::EMBED_ENTITY.to_owned(),
            hint: mxp::Send::EMBED_ENTITY.to_owned(),
            expire: None,
            send_to: SendTo::World,
            menu: false,
        }
    }
}

impl From<mxp::Hyperlink> for Link {
    fn from(value: mxp::Hyperlink) -> Self {
        Self {
            menu: false,
            href: value.href,
            hint: value.hint,
            expire: value.expire,
            send_to: SendTo::Internet,
        }
    }
}

impl From<mxp::Send> for Link {
    fn from(value: mxp::Send) -> Self {
        Self {
            menu: value.is_menu(),
            href: value.href,
            hint: value.hint,
            expire: value.expire,
            send_to: if value.prompt {
                SendTo::Prompt
            } else {
                SendTo::World
            },
        }
    }
}

impl From<&str> for Link {
    fn from(value: &str) -> Self {
        Self {
            href: value.to_owned(),
            hint: value.to_owned(),
            ..Default::default()
        }
    }
}

impl Link {
    /// See [`Send::menu`](mxp::Send::menu).
    pub fn menu(&self) -> mxp::SendMenu<'_> {
        self.as_send().menu()
    }

    /// Returns the tooltip to display when a user mouses over this link.
    /// This is always `self.hint` unless `self.menu` is true.
    pub fn tooltip(&self) -> &str {
        if !self.menu {
            return &self.hint;
        }
        match self.hint.split_once('|') {
            Some((tooltip, _)) => tooltip,
            None => &self.hint,
        }
    }

    /// See [`Send::for_text`](mxp::Send::for_text).
    #[must_use = "function returns a new link"]
    pub fn for_text(&self, text: &str) -> Self {
        if self.send_to == SendTo::Internet {
            return self.clone();
        }
        Self {
            menu: self.menu,
            send_to: self.send_to,
            ..Self::from(self.as_send().for_text(text))
        }
    }

    fn as_send(&self) -> mxp::Send<&str> {
        mxp::Send {
            href: self.href.as_str(),
            hint: self.hint.as_str(),
            expire: self.expire.as_deref(),
            prompt: self.send_to == SendTo::Prompt,
        }
    }
}
