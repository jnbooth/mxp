use std::io::{self, Write};

use mud_transformer::{OutputFragment, TransformerConfig};

pub fn get_config() -> TransformerConfig {
    TransformerConfig {
        terminal_identification: "mushclient".to_owned(),
        ..Default::default()
    }
}

pub fn write_output<I, W>(output: I, mut writer: W) -> io::Result<()>
where
    I: Iterator<Item = OutputFragment>,
    W: Write,
{
    for fragment in output {
        match fragment {
            OutputFragment::Text(fragment) => {
                write!(writer, "{}", fragment)?;
            }
            OutputFragment::LineBreak => {
                writer.write_all(b"\n")?;
            }
            _ => (),
        }
    }
    Ok(())
}
