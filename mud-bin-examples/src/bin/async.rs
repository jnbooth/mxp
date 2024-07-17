use std::io::{self, Read};
use std::thread::{self, JoinHandle};

use bytes::{Bytes, BytesMut};
use mud_bin_examples::write_output;
use mud_stream::nonblocking::MudStream;
use mud_transformer::TransformerConfig;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream as AsyncTcpStream;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> io::Result<()> {
    let stream = AsyncTcpStream::connect(("discworld.atuin.net", 4242)).await?;
    let mut stream = MudStream::new(stream, TransformerConfig::new());
    let mut stdout = io::stdout();
    let (tx_input, mut rx_input) = mpsc::channel(10);
    let input_handle = spawn_input(tx_input);
    loop {
        if input_handle.is_finished() {
            return match input_handle.join() {
                Ok(result) => result,
                Err(_err) => Err(io::Error::new(io::ErrorKind::BrokenPipe, "input panicked")),
            };
        }
        let input = tokio::select! {
            input = rx_input.recv() => input,
            output = stream.read() => match output? {
                Some(output) => {
                    write_output(output, &mut stdout)?;
                    continue;
                }
                None => return Ok(())
            }
        };
        if let Some(input) = input {
            stream.write_all(&input).await?;
        }
    }
}

fn spawn_input(tx_input: mpsc::Sender<Bytes>) -> JoinHandle<io::Result<()>> {
    thread::spawn(move || {
        tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .build()
            .expect("Failed building the input Runtime")
            .block_on(handle_input(tx_input))
    })
}

async fn handle_input(tx: mpsc::Sender<Bytes>) -> io::Result<()> {
    let mut bytes = BytesMut::with_capacity(1024);
    let mut buf = [0; 1024];
    let mut stdin = io::stdin().lock();
    loop {
        let n = stdin.read(&mut buf)?;
        if n == 0 {
            return Ok(());
        }
        bytes.extend_from_slice(&buf[..n]);
        tx.send(bytes.split().freeze())
            .await
            .map_err(|_| io::ErrorKind::BrokenPipe)?;
    }
}
