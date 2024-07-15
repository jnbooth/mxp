use std::io;

use mud_bin_examples::write_output;
use mud_stream::nonblocking::MudStream;
use mud_transformer::TransformerConfig;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream as AsyncTcpStream;

#[tokio::main]
async fn main() -> io::Result<()> {
    let stream = AsyncTcpStream::connect(("discworld.atuin.net", 4242)).await?;
    let mut stream = MudStream::new(stream, TransformerConfig::new());
    let mut stdout = io::stdout();
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
