use mud_stream::nonblocking::MudStream;
use mud_transformer::EffectFragment;
use mud_transformer::{OutputFragment, TextFragment, TextStyle};
use mxp::WorldColor;
use std::io;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use tokio::sync::mpsc;
use tokio::task::JoinHandle;

#[swift_bridge::bridge]
mod ffi {
    enum MudColor {
        Ansi(u8),
        Hex(u32),
    }

    extern "Rust" {
        type RustTextFragment;
        fn text(&self) -> &[u8];
        fn foreground(&self) -> MudColor;
        fn background(&self) -> MudColor;
        fn is_blink(&self) -> bool;
        fn is_bold(&self) -> bool;
        fn is_highlight(&self) -> bool;
        fn is_inverse(&self) -> bool;
        fn is_italic(&self) -> bool;
        fn is_strikeout(&self) -> bool;
        fn is_underline(&self) -> bool;
    }

    enum EffectFragment {
        Backspace,
        Beep,
        CarriageReturn,
        EraseCharacter,
        EraseLine,
    }

    enum OutputFragment {
        Effect(EffectFragment),
        Hr,
        Image(String),
        PageBreak,
        Text(RustTextFragment),
    }

    extern "Rust" {
        type RustMudBridge;
        #[swift_bridge(init)]
        fn new(address: String, port: u16) -> RustMudBridge;
        fn is_connected(&self) -> bool;
        async fn connect(&mut self) -> Result<(), String>;
        fn disconnect(&self) -> bool;
        async fn get_output(&mut self) -> Result<OutputFragment, String>;
        fn send_input(&self, input: String) -> Result<(), String>;
        async fn wait_until_done(&mut self) -> Result<(), String>;
    }
}

pub struct RustMudBridge {
    address: String,
    port: u16,
    handle: Option<JoinHandle<io::Result<()>>>,
    input: mpsc::UnboundedSender<String>,
    output: mpsc::UnboundedReceiver<OutputFragment>,
    output_dispatch: mpsc::UnboundedSender<OutputFragment>,
}

impl RustMudBridge {
    fn new(address: String, port: u16) -> Self {
        let (input, _) = mpsc::unbounded_channel();
        let (output_dispatch, output) = mpsc::unbounded_channel();
        Self {
            address,
            port,
            handle: None,
            input,
            output,
            output_dispatch,
        }
    }

    fn is_connected(&self) -> bool {
        match &self.handle {
            Some(handle) => !handle.is_finished(),
            None => false,
        }
    }

    async fn connect(&mut self) -> Result<(), String> {
        let stream = TcpStream::connect((self.address.clone(), self.port))
            .await
            .map_err(|e| e.to_string())?;
        let (tx_input, mut rx_input) = mpsc::unbounded_channel();
        self.input = tx_input;
        let tx_output = self.output_dispatch.clone();
        self.handle = Some(tokio::spawn(async move {
            let mut stream = MudStream::new(stream, Default::default());
            loop {
                let input = tokio::select! {
                    input = rx_input.recv() => input,
                    fragments = stream.read() => match fragments? {
                        Some(fragments) => {
                            for fragment in fragments {
                                tx_output.send(fragment).map_err(|_| io::ErrorKind::BrokenPipe)?;
                            }
                            continue;
                        }
                        None => return Ok(()),
                    }
                };

                if let Some(input) = input {
                    stream.write_all(input.as_ref()).await?;
                }
            }
        }));
        Ok(())
    }

    pub fn disconnect(&self) -> bool {
        match &self.handle {
            Some(handle) if !handle.is_finished() => {
                handle.abort();
                true
            }
            _ => false,
        }
    }

    pub async fn get_output(&mut self) -> Result<ffi::OutputFragment, String> {
        match self.output.recv().await {
            Some(output) => Ok(output.into()),
            None => Err(io::ErrorKind::BrokenPipe.to_string()),
        }
    }

    pub fn send_input(&self, input: String) -> Result<(), String> {
        self.input.send(input).map_err(|e| e.to_string())
    }

    pub async fn wait_until_done(&mut self) -> Result<(), String> {
        let handle = match self.handle.take() {
            Some(handle) => handle,
            None => return Ok(()),
        };
        match handle.await {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => Err(e.to_string()),
            Err(e) => Err(e.to_string()),
        }
    }
}

impl ffi::MudColor {
    #[inline]
    const fn new(color: WorldColor) -> Self {
        match color {
            mxp::WorldColor::Ansi(code) => Self::Ansi(code),
            mxp::WorldColor::Hex(color) => Self::Hex(color.code()),
        }
    }
}

impl From<WorldColor> for ffi::MudColor {
    #[inline]
    fn from(value: WorldColor) -> Self {
        match value {
            mxp::WorldColor::Ansi(code) => Self::Ansi(code),
            mxp::WorldColor::Hex(color) => Self::Hex(color.code()),
        }
    }
}

#[repr(transparent)]
struct RustTextFragment {
    inner: TextFragment,
}

macro_rules! flag_method {
    ($n:ident, $v:expr) => {
        #[inline]
        pub fn $n(&self) -> bool {
            self.inner.flags.contains($v)
        }
    };
}

impl RustTextFragment {
    #[inline]
    fn text(&self) -> &[u8] {
        // SAFETY: Text fragments are UTF-8 validated during transformation.
        // swift_bridge doesn't have a type for &[u8] anyway.
        &self.inner.text
    }

    #[inline]
    const fn foreground(&self) -> ffi::MudColor {
        ffi::MudColor::new(self.inner.foreground)
    }

    #[inline]
    const fn background(&self) -> ffi::MudColor {
        ffi::MudColor::new(self.inner.background)
    }

    flag_method!(is_blink, TextStyle::Blink);
    flag_method!(is_bold, TextStyle::Bold);
    flag_method!(is_highlight, TextStyle::Highlight);
    flag_method!(is_inverse, TextStyle::Inverse);
    flag_method!(is_italic, TextStyle::Italic);
    flag_method!(is_strikeout, TextStyle::Strikeout);
    flag_method!(is_underline, TextStyle::Underline);
}

impl From<EffectFragment> for ffi::EffectFragment {
    fn from(value: EffectFragment) -> Self {
        match value {
            EffectFragment::Backspace => Self::Backspace,
            EffectFragment::Beep => Self::Beep,
            EffectFragment::CarriageReturn => Self::CarriageReturn,
            EffectFragment::EraseCharacter => Self::EraseCharacter,
            EffectFragment::EraseLine => Self::EraseLine,
        }
    }
}

impl From<OutputFragment> for ffi::OutputFragment {
    fn from(value: OutputFragment) -> Self {
        match value {
            OutputFragment::Effect(effect) => Self::Effect(effect.into()),
            OutputFragment::Hr => Self::Hr,
            OutputFragment::Image(src) => Self::Image(src),
            OutputFragment::PageBreak => Self::PageBreak,
            OutputFragment::Text(text) => Self::Text(RustTextFragment { inner: text }),
        }
    }
}