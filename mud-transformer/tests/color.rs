mod common;
use common::transform;
use mud_transformer::TextFragment;
use mxp::RgbColor;

#[test]
fn basic_color() {
    let output = transform("\x1B[31mRed\x1B[39;49m\x1B[0m,").output();
    let expected = &[
        TextFragment {
            text: "Red".into(),
            foreground: Some(RgbColor::hex(0x800000)),
            ..Default::default()
        }
        .into(),
        TextFragment::from(",").into(),
    ];
    assert_eq!(output, expected);
}

#[test]
fn xterm_color() {
    let output = transform("\x1B[38;5;34m DarkGreen\x1B[39;49m\x1B[0m,").output();
    let expected = &[
        TextFragment {
            text: " DarkGreen".into(),
            foreground: Some(RgbColor::hex(0x00AF00)),
            ..Default::default()
        }
        .into(),
        TextFragment::from(",").into(),
    ];
    assert_eq!(output, expected);
}

#[test]
fn mxp_color() {
    let output = transform("\x1B[4z<C #D75FAF>a\x1B[0m,\x1B[31mb").output();
    let expected = &[
        TextFragment {
            text: "a".into(),
            foreground: Some(RgbColor::hex(0xD75FAF)),
            ..Default::default()
        }
        .into(),
        TextFragment::from(",").into(),
        TextFragment {
            text: "b".into(),
            foreground: Some(RgbColor::hex(0x800000)),
            ..Default::default()
        }
        .into(),
    ];
    assert_eq!(output, expected);
}

/*
#[test]
fn ansi_color() {
    let output = transform(include_str!("samples/colors.ansi")).output();
    for fragment in output {
        match fragment {
            OutputFragment::Text(fragment) => {
                print!("{}", fragment.ansi());
            }
            OutputFragment::LineBreak => println!(),
            _ => (),
        }
    }
}
 */
