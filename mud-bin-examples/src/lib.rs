use std::io::{self, Write};

use mud_transformer::OutputFragment;

pub fn write_output<I, W>(output: I, mut writer: W) -> io::Result<()>
where
    I: Iterator<Item = OutputFragment>,
    W: Write,
{
    for fragment in output {
        if let Some(bytes) = fragment.as_bytes() {
            writer.write_all(bytes)?;
        }
    }
    Ok(())
}
