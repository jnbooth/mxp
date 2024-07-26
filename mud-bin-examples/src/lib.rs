use std::io::{self, Write};

use mud_transformer::OutputFragment;

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
