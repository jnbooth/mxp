use bytestring::ByteString;
use bytestringmut::ByteStringMut;

pub trait ByteStringMutExt {
    fn share(&mut self, s: &str) -> ByteString;
}

impl ByteStringMutExt for ByteStringMut {
    fn share(&mut self, s: &str) -> ByteString {
        self.clear();
        self.push_str(s);
        self.split().freeze()
    }
}
