mod common;
use common::transform;
use mud_transformer::TextFragment;

#[test]
fn basic_link() {
    let output =
        transform("&lt;\x1B[4z<send href=\"options mxp alias\">more options\x1B[4z</send>&gt;")
            .output();
    let expected = &[
        TextFragment::from("<").into(),
        TextFragment {
            text: "more options".into(),
            action: Some(mxp::Link {
                action: "options mxp alias".to_owned(),
                ..Default::default()
            }),
            ..Default::default()
        }
        .into(),
        TextFragment::from(">").into(),
    ];
    assert_eq!(output, expected);
}

#[test]
fn escape_in_link() {
    let output =
        transform("\x1B[4z<send href=\"options mxp livingmenus=off\">off\x1B[4z</send>").output();
    let expected = &[TextFragment {
        text: "off".into(),
        action: Some(mxp::Link {
            action: "options mxp livingmenus=off".into(),
            ..Default::default()
        }),
        ..Default::default()
    }
    .into()];
    assert_eq!(output, expected);
}
