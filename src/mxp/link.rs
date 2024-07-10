use enumeration::Enum;

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

impl SendTo {
    pub fn attach(self, s: &str) -> String {
        match self {
            Self::World => ["send:", s].concat(),
            Self::Input => ["echo:", s].concat(),
            _ if s.starts_with("echo:") || s.starts_with("send:") => ["http://", s].concat(),
            Self::Internet => s.to_owned(),
        }
    }

    pub fn detach(s: &str) -> (Self, &str) {
        if let Some(world) = s.strip_prefix("send:") {
            (Self::World, world)
        } else if let Some(input) = s.strip_prefix("echo:") {
            (Self::Input, input)
        } else {
            (Self::Internet, s)
        }
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
}

impl Link {
    pub fn new(action: &str, hint: Option<&str>, sendto: SendTo) -> Self {
        let mut actions = action.split('|');
        let action = sendto.attach(actions.next().unwrap());
        match hint {
            None => Self {
                action,
                hint: None,
                prompts: actions.map(ToOwned::to_owned).collect(),
                sendto,
            },
            Some(hint) => {
                let mut hints = hint.split('|').map(ToOwned::to_owned);
                let first_hint = hints.next().unwrap();
                let mut prompts: Vec<_> = hints.collect();
                if prompts.is_empty() {
                    prompts = actions.map(ToOwned::to_owned).collect();
                }
                Self {
                    action,
                    hint: Some(first_hint),
                    prompts,
                    sendto,
                }
            }
        }
    }
}
