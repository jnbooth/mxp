use std::io::{self, Write};

use crate::escape::telnet;

/// [RFC 859](https://datatracker.ietf.org/doc/html/rfc859): STATUS
pub const OPT: u8 = 5;

pub const IS: u8 = 0;
pub const SEND: u8 = 1;

pub fn encode<W, I>(mut writer: W, verb: u8, iter: I) -> io::Result<()>
where
    W: Write,
    I: IntoIterator<Item = u8>,
{
    for opt in iter {
        writer.write_all(&[verb, opt])?;
        if opt == telnet::SE {
            writer.write_all(&[opt])?;
        }
    }
    Ok(())
}
