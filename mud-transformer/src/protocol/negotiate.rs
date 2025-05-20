use crate::TransformerConfig;

pub(crate) trait Negotiate {
    const CODE: u8;

    fn negotiate(self, buf: &mut Vec<u8>, config: &TransformerConfig);
}
