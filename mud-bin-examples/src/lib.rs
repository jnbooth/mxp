#[macro_use]
extern crate enumeration;

use std::io::{self, Write};

use mud_transformer::mxp::ActionKind;
use mud_transformer::{Output, OutputFragment, TransformerConfig};

pub fn get_config() -> TransformerConfig {
    TransformerConfig {
        terminal_identification: "mushclient".to_owned(),
        unsupported_actions: enums![
            ActionKind::Center,
            ActionKind::Sound,
            ActionKind::Music,
            ActionKind::Image,
            ActionKind::Send,
            ActionKind::Relocate,
            ActionKind::Frame,
            ActionKind::Dest,
            ActionKind::Filter,
            ActionKind::Hyperlink,
            ActionKind::H1,
            ActionKind::H2,
            ActionKind::H3,
            ActionKind::H4,
            ActionKind::H5,
            ActionKind::H6,
            ActionKind::Hr,
            ActionKind::Strikeout,
            ActionKind::Script,
            ActionKind::Small,
            ActionKind::Tt,
            ActionKind::Samp,
            ActionKind::Var,
            ActionKind::Gauge,
            ActionKind::Stat,
            ActionKind::Expire,
            ActionKind::Reset
        ],
        ..Default::default()
    }
}

pub fn write_output<I, W>(iter: I, mut writer: W) -> io::Result<()>
where
    I: Iterator<Item = Output>,
    W: Write,
{
    for output in iter {
        match output.fragment {
            OutputFragment::Text(fragment) => {
                write!(writer, "{fragment}")?;
            }
            OutputFragment::LineBreak => {
                writer.write_all(b"\n")?;
            }
            _ => (),
        }
    }
    Ok(())
}
