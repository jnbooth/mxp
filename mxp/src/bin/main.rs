use std::io::{self, Write};

use mxp::{OutputFragment, TransformerConfig};

#[cfg(not(feature = "async"))]
fn main() -> io::Result<()> {
    use std::net::TcpStream as SyncTcpStream;
    let stream = SyncTcpStream::connect(("discworld.atuin.net", 4242))?;
    let mut stream = mxp::sync::MudStream::new(stream, TransformerConfig::new());
    let mut stdout = io::stdout().lock();
    while let Some(output) = stream.read()? {
        write_output(output, &mut stdout)?;
    }

    Ok(())
}

#[cfg(feature = "async")]
#[tokio::main]
async fn main() -> io::Result<()> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream as AsyncTcpStream;

    let stream = AsyncTcpStream::connect(("discworld.atuin.net", 4242)).await?;
    let mut stream = mxp::tokio::MudStream::new(stream, TransformerConfig::new());
    let mut stdout = io::stdout().lock();
    let mut stdin = tokio::io::stdin();
    let mut input = [0; 1024];
    loop {
        let n = tokio::select! {
            n = stdin.read(&mut input) => n?,
            output = stream.read() => match output? {
                Some(output) => {
                    write_output(output, &mut stdout)?;
                    continue;
                }
                None => return Ok(())
            }
        };
        stream.write_all(&input[0..n]).await?;
    }
}

fn write_output<I, W>(output: I, mut writer: W) -> io::Result<()>
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
