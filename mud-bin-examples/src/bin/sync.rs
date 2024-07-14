use std::io;

use mud_bin_examples::write_output;
use mud_stream::blocking::MudStream;
use mud_transformer::TransformerConfig;

fn main() -> io::Result<()> {
    use std::net::TcpStream as SyncTcpStream;
    let stream = SyncTcpStream::connect(("discworld.atuin.net", 4242))?;
    let mut stream = MudStream::new(stream, TransformerConfig::new());
    let mut stdout = io::stdout().lock();
    while let Some(output) = stream.read()? {
        write_output(output, &mut stdout)?;
    }

    Ok(())
}
