use std::io::{self, Write};

use mud_transformer::{Output, OutputFragment, TransformerConfig};

pub fn get_config() -> TransformerConfig {
    TransformerConfig {
        terminal_identification: "mushclient".to_owned(),
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
