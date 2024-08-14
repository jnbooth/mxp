use mud_bin_examples::{get_config, write_output};
use mud_stream::blocking::MudStream;
use std::io;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

fn main() -> io::Result<()> {
    let stream = TcpStream::connect(("discworld.atuin.net", 4242))?;
    stream.set_read_timeout(Some(Duration::from_secs(3)))?;
    let mut stream = MudStream::new(stream, get_config());
    let mut stdin = io::stdin().lock();
    let mut stdout = io::stdout();
    let mut buf = [0; 1024];

    loop {
        let input = match stream.read() {
            Ok(Some(output)) => {
                write_output(output, &mut stdout)?;
                None
            }
            Ok(None) => {
                return Ok(());
            }
            Err(e)
                if e.kind() == io::ErrorKind::WouldBlock || e.kind() == io::ErrorKind::TimedOut =>
            {
                let n = stdin.read(&mut buf)?;
                if n == 0 {
                    return Ok(());
                }
                Some(&buf[..n])
            }
            Err(e) => {
                return Err(e);
            }
        };
        if let Some(input) = input {
            stream.write_all(input)?;
        }
    }
}
