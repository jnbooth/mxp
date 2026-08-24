mod common;
use common::transform;
use mud_transformer::output::{OutputFragment, TextFragment};

#[test]
fn left_quote() {
    let line = "U+2018 ‘ and U+2019 ’\n";
    let output = transform(line).output();
    let expected = &[
        TextFragment::from(&line[..line.len() - 1]).into(),
        OutputFragment::LineBreak,
    ];
    assert_eq!(output, expected);
}
