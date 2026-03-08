mod common;
use common::transform;
use mud_transformer::output::TextFragment;

#[test]
fn basic_entity() {
    let output = transform("&lt;&gt;").output();
    let expected = &[TextFragment::from("<>").into()];
    assert_eq!(output, expected);
}
