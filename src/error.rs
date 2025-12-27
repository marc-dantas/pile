use crate::{
    cli::*,
    lexer::FileSpan,
    parser::ParseError,
    runtime::RuntimeError,
    CLIError,
};

const RED: &'static str = "\x1B[1;31m";
const GREEN: &'static str = "\x1B[1;32m";
const CYAN: &'static str = "\x1B[1;35m";
const RESET: &'static str = "\x1B[0m";

fn match_runtime_error(e: &RuntimeError) {
    match e {
        RuntimeError::Custom(span, message) => {
            throw(
                "runtime error",
                &format!("{message}"),
                span,
                None,
            );
        }
        RuntimeError::ArrayOutOfBounds(span, index, len) => {
            throw(
                "runtime error",
                &format!("array index out of bounds: tried to index array of size {len} but used index {index}."),
                span,
                None,
            );
        }
        RuntimeError::StringOutOfBounds(span, index, len) => {
            throw(
                "runtime error",
                &format!("string index out of bounds: tried to index string of size {len} but used index {index}."),
                span,
                None,
            );
        }
        RuntimeError::InvalidSymbol(span, x) => {
            throw(
                "runtime error",
                &format!("invalid symbol: `{x}` is not defined."),
                span,
                Some("maybe a typo?"),
            );
        }
        RuntimeError::EmptyDefinition(span, x) => {
            throw(
                "runtime error",
                &format!("found empty definition: the expression inside {x} leads to no value on the stack."),
                span,
                None,
            );
        }
        RuntimeError::StackUnderflow(span, op, n) => {
            throw(
                "runtime error",
                &format!("stack underflow: too few values on the stack to satisfy `{op}` (expected {n})"),
                span,
                Some(&format!("use `trace` operation to see the values on the stack without removing them.")),
            );
        }
        RuntimeError::UnexpectedType(span, n, x, y) => {
            throw(
                "runtime error",
                &format!(
                    "unexpected type: `{n}` expects {x} on the stack to work, but got {y}."
                ),
                span,
                Some("try checking the values before the operation."),
            );
        }
        RuntimeError::DivisionByZero(span) => {
            throw(
                "runtime error",
                &format!("division by zero."),
                span,
                None,
            );
        }
    }
}

pub fn runtime_error(e: RuntimeError) {
    match e {
        x => match_runtime_error(&x),
    }
}

pub fn parse_error(e: ParseError) {
    match e {
        ParseError::UnmatchedBlock(span) => {
            throw(
                "parse error",
                "unmatched block: termination of block (`end`) provided without a beginning.",
                &vec![span],
                None,
            );
        }
        ParseError::UnterminatedBlock(span, x) => {
            throw(
                "parse error",
                &format!("unterminated block: termination of block not provided from `{x}` block."),
                &vec![span],
                Some("perhaps you forgot to write `end`?"),
            );
        }
        ParseError::UnexpectedEOF(span, x) => {
            throw(
                "parse error",
                &format!(
                    "unexpected end of file: expected {x} but got the end of the file (nothing)."
                ),
                &vec![span],
                None,
            );
        }
        ParseError::UnexpectedToken(span, x, y) => {
            throw(
                "parse error",
                &format!("unexpected token: expected {y} but got {x}."),
                &vec![span],
                None,
            );
        }
    };
}

pub fn cli_error(e: CLIError) {
    show_usage();
    show_help();
    match e {
        CLIError::InvalidFlag(x) => {
            fatal(&format!("invalid flag \"{x}\""));
        }
        CLIError::ExpectedArgument(x) => {
            fatal(&format!("expected positional argument {x}"));
        }
        CLIError::UnexpectedArgument(x) => {
            fatal(&format!("found unexpected argument \"{x}\""));
        }
    }
}

pub fn fatal(message: &str) -> ! {
    eprintln!("pile: fatal: {message}");
    std::process::exit(1);
}

pub fn throw(
    error: &str,
    message: &str,
    call_stack: &[FileSpan],
    help: Option<&str>,
) {
    eprintln!("pile: {RED}{}{RESET}:", error);

    for span in call_stack {
        eprintln!(" {CYAN}->{RESET} {}:{}:{}", span.filename, span.line, span.col);
    }
    for line in break_line_at(message.to_string(), 50) {
        eprintln!("      {line}");
    }
    if let Some(h) = help {
        for line in break_line_at(h.to_string(), 50) {
            eprintln!(" {GREEN} +   {}{RESET}", line);
        }
    }
    eprintln!();
    std::process::exit(1);
}

fn break_line_at(value: String, n: usize) -> Vec<String> {
    let mut line = String::new();
    let words = value.split(|x: char| x.is_whitespace());
    let mut lines = Vec::new();
    for w in words {
        line.push_str(&format!("{w} "));
        if line.len() + w.len() + 1 > n {
            lines.push(line.clone());
            line = String::new();
        }
    }
    if line.len() > 0 {
        lines.push(line.clone());
    }
    lines
}
