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
    pub fn new(action: &str, hints: Option<&str>, sendto: SendTo) -> Self {
        let (action, actions) = split_list(action);
        match hints {
            None => Self {
                action,
                hint: None,
                prompts: actions,
                sendto,
            },
            Some(hints) => {
                let (hint, prompts) = split_list(hints);
                Self {
                    action,
                    hint: Some(hint),
                    prompts: if prompts.is_empty() { actions } else { prompts },
                    sendto,
                }
            }
        }
    }
}

fn split_list(list: &str) -> (String, Vec<String>) {
    let mut iter = list.split('|');
    let first = iter.next().unwrap().to_owned();
    (first, iter.map(ToOwned::to_owned).collect())
}
