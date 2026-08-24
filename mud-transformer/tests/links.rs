mod common;
use common::transform;
use mud_transformer::output::{Link, SendTo, TextFragment};

#[test]
fn anchor_embed_text_entity() {
    let output = transform("\x1B[4z<a href=&text;>https://youtube.com\x1B[4z</a>").output();
    let expected = &[TextFragment {
        text: "https://youtube.com".into(),
        link: Some(Link {
            send_to: SendTo::Internet,
            .."https://youtube.com".into()
        }),
        ..Default::default()
    }
    .into()];
    assert_eq!(output, expected);
}

#[test]
fn basic_link() {
    let output =
        transform("&lt;\x1B[4z<send href=\"options mxp alias\">more options\x1B[4z</send>&gt;")
            .output();
    let expected = &[
        TextFragment::from("<").into(),
        TextFragment {
            text: "more options".into(),
            link: Some("options mxp alias".into()),
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
        link: Some("options mxp livingmenus=off".into()),
        ..Default::default()
    }
    .into()];
    assert_eq!(output, expected);
}
