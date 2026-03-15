mod common;
use common::{transform, transform_with};
use mud_transformer::output::{Link, TextFragment, TextStyle};
use mud_transformer::{TransformerConfig, UseMxp};

#[test]
fn mslp_underline() {
    let config = TransformerConfig {
        use_mxp: UseMxp::Always,
        linkify_underlined: true,
        ..Default::default()
    };
    let output = transform_with(config, "\x1B[4mTest link\x1B[0m").output();
    let expected = &[TextFragment {
        text: "Test link".into(),
        flags: TextStyle::Underline.into(),
        link: Some("Test link".into()),
        ..Default::default()
    }
    .into()];
    assert_eq!(output, expected);
}

#[test]
fn mslp_deunderlined() {
    let config = TransformerConfig {
        use_mxp: UseMxp::Always,
        linkify_underlined: true,
        ..Default::default()
    };
    let output = transform_with(config, "\x1B[4;24mTest link\x1B[0m").output();
    let expected = &[TextFragment {
        text: "Test link".into(),
        link: Some("Test link".into()),
        ..Default::default()
    }
    .into()];
    assert_eq!(output, expected);
}

#[test]
fn mslp_underline_disabled() {
    let config = TransformerConfig {
        use_mxp: UseMxp::Always,
        linkify_underlined: false,
        ..Default::default()
    };
    let output = transform_with(config, "\x1B[4mTest link\x1B[0m").output();
    let expected = &[TextFragment {
        text: "Test link".into(),
        flags: TextStyle::Underline.into(),
        ..Default::default()
    }
    .into()];
    assert_eq!(output, expected);
}

#[test]
fn mslp_menu() {
    let output = transform("\x1B]68;1;MENU;{a tasty donut}{buy donut} {a loaf of bread}{buy bread} {a big tomato}{buy tomato}\x07\x1B[4mshopping list\x1B[24m").output();
    let expected = &[TextFragment {
        text: "shopping list".into(),
        flags: TextStyle::Underline.into(),
        link: Some(Link {
            href: "buy donut|buy bread|buy tomato".into(),
            hint: "a tasty donut|a loaf of bread|a big tomato".into(),
            ..Default::default()
        }),
        ..Default::default()
    }
    .into()];
    assert_eq!(output, expected);
}

#[test]
fn mslp_send() {
    let output = transform("\x1B]68;1;SEND;say Hello World!\x07\x1B[4m(click me)\x1B[24m").output();
    let expected = &[TextFragment {
        text: "(click me)".into(),
        flags: TextStyle::Underline.into(),
        link: Some("say Hello World!".into()),
        ..Default::default()
    }
    .into()];
    assert_eq!(output, expected);
}

#[test]
fn mslp_spaced() {
    let output =
        transform("\x1B]68;1;SEND;say Hello World!\x07 \x1B[4m(click me)\x1B[24m").output();
    let expected = &[
        TextFragment::from(" ").into(),
        TextFragment {
            text: "(click me)".into(),
            flags: TextStyle::Underline.into(),
            ..Default::default()
        }
        .into(),
    ];
    assert_eq!(output, expected);
}

#[test]
fn mslp_escaped() {
    let output =
        transform("\x1B]68;1;SEND;say Hello World!\x07\x1B[0m\x1B[4m(click me)\x1B[24m").output();
    let expected = &[TextFragment {
        text: "(click me)".into(),
        flags: TextStyle::Underline.into(),
        ..Default::default()
    }
    .into()];
    assert_eq!(output, expected);
}
