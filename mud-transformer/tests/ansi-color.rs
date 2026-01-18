use mud_transformer::{OutputFragment, TextFragment, Transformer};
use mxp::RgbColor;

fn transform(bytes: &[u8]) -> Vec<OutputFragment> {
    let mut transformer = Transformer::default();
    let mut buf = vec![0; 1024 * 20];
    transformer.receive(bytes, &mut buf).unwrap();
    transformer
        .flush_output()
        .map(|output| output.fragment)
        .collect()
}

#[test]
fn ansi_red() {
    let output = transform(include_bytes!("samples/red.ansi"));
    let expected: &[OutputFragment] = &[
        OutputFragment::Text(TextFragment {
            text: "Red".into(),
            foreground: Some(RgbColor { r: 128, g: 0, b: 0 }),
            ..Default::default()
        }),
        OutputFragment::Text(TextFragment {
            text: ",".into(),
            ..Default::default()
        }),
    ];
    assert_eq!(output, expected);
}

#[test]
fn ansi_darkgreen() {
    let output = transform(include_bytes!("samples/darkgreen.ansi"));
    let expected: &[OutputFragment] = &[
        OutputFragment::Text(TextFragment {
            text: " DarkGreen".into(),
            foreground: Some(RgbColor { r: 0, g: 175, b: 0 }),
            ..Default::default()
        }),
        OutputFragment::Text(TextFragment {
            text: ",".into(),
            ..Default::default()
        }),
    ];
    assert_eq!(output, expected);
}

#[test]
fn ansi_color() {
    let output = transform(include_bytes!("samples/colors.ansi"));
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
