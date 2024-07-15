use bytes::BytesMut;

pub trait Output {
    fn output(self, buf: &mut BytesMut);
}

impl Output for &[u8] {
    #[inline]
    fn output(self, buf: &mut BytesMut) {
        buf.extend_from_slice(self)
    }
}

impl<const N: usize> Output for &[u8; N] {
    #[inline]
    fn output(self, buf: &mut BytesMut) {
        buf.extend_from_slice(self)
    }
}

impl Output for &mut Vec<u8> {
    #[inline]
    fn output(self, buf: &mut BytesMut) {
        buf.extend_from_slice(&self)
    }
}

impl Output for &str {
    #[inline]
    fn output(self, buf: &mut BytesMut) {
        buf.extend_from_slice(self.as_bytes())
    }
}

impl Output for &String {
    #[inline]
    fn output(self, buf: &mut BytesMut) {
        buf.extend_from_slice(self.as_bytes())
    }
}

impl Output for u8 {
    #[inline]
    fn output(self, buf: &mut BytesMut) {
        buf.extend_from_slice(&[self])
    }
}
