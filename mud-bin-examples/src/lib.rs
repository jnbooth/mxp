#[macro_use]
extern crate enumeration;

use std::io::{self, Write};

use mud_transformer::{Output, OutputFragment, Tag, TransformerConfig};

pub fn get_config() -> TransformerConfig {
    TransformerConfig {
        terminal_identification: "mushclient".to_owned(),
        supports: enums![
            Tag::Bold,
            Tag::Color,
            Tag::Italic,
            Tag::Send,
            Tag::Strikeout,
            Tag::Underline,
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
            OutputFragment::MxpError(e) => {
                writeln!(writer, "\nMXP error: {e}")?;
            }
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
