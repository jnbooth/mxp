use std::io::Read;
use std::ops::{Deref, DerefMut};

use mud_transformer::{OutputFragment, Transformer, TransformerConfig, UseMxp};

pub fn transform<S: AsRef<[u8]>>(s: S) -> TestTransformer {
    let config = TransformerConfig {
        use_mxp: UseMxp::Always,
        ..Default::default()
    };
    transform_with(config, s)
}

pub fn transform_with<S: AsRef<[u8]>>(config: TransformerConfig, s: S) -> TestTransformer {
    let mut transformer = Transformer::new(config);
    let mut buf = [0; 1024 * 10];
    transformer.receive(s.as_ref(), &mut buf).unwrap();
    TestTransformer { transformer }
}

pub struct TestTransformer {
    transformer: Transformer,
}

impl Deref for TestTransformer {
    type Target = Transformer;

    fn deref(&self) -> &Self::Target {
        &self.transformer
    }
}

impl DerefMut for TestTransformer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.transformer
    }
}

impl TestTransformer {
    #[track_caller]
    pub fn input(&mut self) -> String {
        let mut input = Vec::new();
        if let Some(mut drain) = self.transformer.drain_input() {
            drain.read_to_end(&mut input).unwrap();
        }
        String::from_utf8(input).expect("invalid UTF-8")
    }

    pub fn output(&mut self) -> Vec<OutputFragment> {
        self.transformer
            .flush_output()
            .map(|output| output.fragment)
            .collect()
    }
}
