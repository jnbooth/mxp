use flate2::read::ZlibDecoder;
use std::io::{self, Cursor, IoSliceMut, Read};
use std::{mem, vec};

use crate::config::COMPRESS_BUFFER;

#[derive(Debug)]
pub enum ZState<R> {
    Prepend(Cursor<Vec<u8>>, R),
    Direct(R),
    Transitioning,
}

impl<R> ZState<R> {
    pub fn into_inner(self) -> R {
        match self {
            Self::Prepend(_, reader) => reader,
            Self::Direct(reader) => reader,
            Self::Transitioning => unreachable!(),
        }
    }

    pub fn get_ref(&self) -> &R {
        match self {
            Self::Prepend(_, reader) => reader,
            Self::Direct(reader) => reader,
            Self::Transitioning => unreachable!(),
        }
    }

    pub fn get_mut(&mut self) -> &mut R {
        match self {
            Self::Prepend(_, reader) => reader,
            Self::Direct(reader) => reader,
            Self::Transitioning => unreachable!(),
        }
    }
}

macro_rules! impl_read {
    ($me:ident, $buft:ty) => {
        fn $me(&mut self, buf: &mut $buft) -> io::Result<usize> {
            let reached = match self {
                Self::Direct(reader) => return reader.$me(buf),
                Self::Prepend(prepend, reader) => {
                    let mut reached = prepend.$me(buf)?;
                    if reached == buf.len() {
                        return Ok(reached);
                    }
                    reached += reader.$me(&mut buf[reached..])?;
                    reached
                }
                Self::Transitioning => unreachable!(),
            };
            let mut transition = Self::Transitioning;
            mem::swap(self, &mut transition);
            *self = Self::Direct(transition.into_inner());
            Ok(reached)
        }
    };
}

impl<R: Read> Read for ZState<R> {
    impl_read!(read, [u8]);
    impl_read!(read_vectored, [IoSliceMut]);
}

#[derive(Debug)]
pub enum DecompressStream<R> {
    Uncompressed(R),
    Compressed(ZlibDecoder<ZState<R>>),
    Transitioning,
}

impl<R> DecompressStream<R> {
    pub fn into_inner(self) -> R {
        match self {
            Self::Uncompressed(reader) => reader,
            Self::Compressed(reader) => reader.into_inner().into_inner(),
            Self::Transitioning => unreachable!(),
        }
    }

    pub const fn new(reader: R) -> Self {
        Self::Uncompressed(reader)
    }

    pub fn get_ref(&self) -> &R {
        match self {
            Self::Uncompressed(stream) => stream,
            Self::Compressed(decompress) => decompress.get_ref().get_ref(),
            Self::Transitioning => unreachable!(),
        }
    }

    pub fn get_mut(&mut self) -> &mut R {
        match self {
            Self::Uncompressed(stream) => stream,
            Self::Compressed(decompress) => decompress.get_mut().get_mut(),
            Self::Transitioning => unreachable!(),
        }
    }

    pub fn reset(&mut self) {
        let mut buf = Self::Transitioning;
        mem::swap(self, &mut buf);
        *self = Self::Uncompressed(buf.into_inner());
    }

    pub fn start_decompressing(&mut self, prepend: Vec<u8>)
    where
        R: Read,
    {
        let mut buf = Self::Transitioning;
        mem::swap(self, &mut buf);
        let reader = buf.into_inner();
        let inner = if prepend.is_empty() {
            ZState::Direct(reader)
        } else {
            ZState::Prepend(Cursor::new(prepend), reader)
        };
        *self = Self::Compressed(ZlibDecoder::new_with_buf(inner, vec![0; COMPRESS_BUFFER]));
    }
}

impl<R: Read> Read for DecompressStream<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::Uncompressed(r) => r.read(buf),
            Self::Compressed(r) => r.read(buf),
            Self::Transitioning => unreachable!(),
        }
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut]) -> io::Result<usize> {
        match self {
            Self::Uncompressed(r) => r.read_vectored(bufs),
            Self::Compressed(r) => r.read_vectored(bufs),
            Self::Transitioning => unreachable!(),
        }
    }
}
