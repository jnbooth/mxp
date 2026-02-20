use crate::input::BufferedInput;
use crate::output::{BufferedOutput, OutputFragment, TextFragment};
use crate::protocol::ansi::Interpreter;

pub fn interpret_ansi(input: &str) -> Vec<TextFragment> {
    let mut iter = input.split("\x1B[");
    let Some(start) = iter.next() else {
        return Vec::new();
    };
    let mut ignored = BufferedInput::new();
    let mut interpreter = Interpreter::new();
    let mut output = BufferedOutput::new();
    output.append_text(start);
    for sequence in iter {
        let Some(end) = sequence.find(|c: char| matches!(c, '@'..='z')) else {
            continue;
        };
        let Some((escape, rest)) = sequence.split_at_checked(end + 1) else {
            continue;
        };
        interpreter.reset();
        for &c in escape.as_bytes() {
            interpreter.interpret(c, &mut output, &mut ignored);
        }
        output.append_text(rest);
    }
    output
        .into_output()
        .into_iter()
        .filter_map(|output| match output.fragment {
            OutputFragment::Text(text) => Some(text),
            _ => None,
        })
        .collect()
}
