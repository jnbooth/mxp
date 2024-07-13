use mxp::{Transformer, TransformerConfig};

fn main() {
    let mut transformer = Transformer::new(TransformerConfig::default());
    transformer.interpret_char(0);
}
