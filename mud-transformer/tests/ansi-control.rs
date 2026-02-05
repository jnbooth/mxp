mod common;
use std::fmt::Write as _;

use common::transform;
use mud_transformer::ControlFragment;

#[test]
fn set_title() {
    let output = transform("\x1B]2;Test title\x1B\\").output();
    let expected = &[ControlFragment::SetTitle("Test title".into()).into()];
    assert_eq!(output, expected);
}

#[test]
fn request_sgr() {
    let mut transformer = transform("\x1BP$qm\x1B\\");
    assert_eq!(transformer.input(), "\x1BP0$r0m\x1B\\");
    assert_eq!(transformer.output(), &[]);
}

#[test]
fn load_answerback() {
    let mut message = "\x1BP1v".to_owned();
    for &c in b"Test answerback" {
        write!(message, "{c:02X}").unwrap();
    }
    message.push_str("\x1B\\\x05");

    let mut transformer = transform(&message);
    assert_eq!(transformer.input(), "Test answerback");
    assert_eq!(transformer.output(), &[]);
}
