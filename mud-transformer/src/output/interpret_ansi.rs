use super::{BufferedOutput, Output, OutputFragment, TextFragment};
use crate::input::BufferedInput;
use crate::protocol::ansi;

fn iter_ansi(output: Vec<Output>) -> impl Iterator<Item = TextFragment> {
    output
        .into_iter()
        .filter_map(|output| match output.fragment {
            OutputFragment::Text(text) => Some(text),
            _ => None,
        })
}

pub fn interpret_ansi(input: &str) -> impl Iterator<Item = TextFragment> {
    let mut iter = input.split("\x1B[");
    let Some(start) = iter.next() else {
        return iter_ansi(Vec::new());
    };
    let mut ignored = BufferedInput::new();
    let mut interpreter = ansi::Interpreter::new();
    let mut output = BufferedOutput::new();
    output.write_str(start);
    for sequence in iter {
        let Some(end) = sequence.find(|c: char| matches!(c, '@'..='z')) else {
            continue;
        };
        let Some((escape, rest)) = sequence.split_at_checked(end + 1) else {
            continue;
        };
        interpreter.reset();
        for c in escape.bytes() {
            interpreter.interpret(c, &mut output, &mut ignored);
        }
        output.write_str(rest);
    }
    iter_ansi(output.into_output())
}
