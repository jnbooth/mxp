use mud_transformer::{OutputFragment, TextFragment, Transformer};
use mxp::RgbColor;

#[test]
fn ansi_red() {
    let mut transformer = Transformer::default();
    let mut buf = vec![0; 1024 * 20];
    transformer
        .receive(include_bytes!("samples/ansi-red"), &mut buf)
        .unwrap();
    let output = transformer
        .flush_output()
        .map(|output| output.fragment)
        .collect::<Vec<_>>();
    let expected: &[OutputFragment] = &[
        OutputFragment::Text(TextFragment {
            text: "Red".into(),
            foreground: RgbColor { r: 128, g: 0, b: 0 },
            ..Default::default()
        }),
        OutputFragment::Text(TextFragment {
            text: ",".into(),
            ..Default::default()
        }),
        OutputFragment::LineBreak,
    ];
    assert_eq!(output, expected);
}

#[test]
fn ansi_color() {
    let mut transformer = Transformer::default();
    let mut buf = vec![0; 1024 * 20];
    transformer
        .receive(include_bytes!("samples/ansi-color"), &mut buf)
        .unwrap();
    for output in transformer.flush_output() {
        match output.fragment {
            OutputFragment::Text(fragment) => {
                print!("{}", fragment.ansi());
            }
            OutputFragment::LineBreak => println!(),
            _ => (),
        }
    }
}
