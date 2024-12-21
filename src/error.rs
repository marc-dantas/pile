use crate::{lexer::TokenSpan, cli::{show_usage, show_help}, parser::ParseError, runtime::RuntimeError, CLIError};

fn match_runtime_error(e: &RuntimeError, call: Option<TokenSpan>) {
    match e {
        RuntimeError::ProcedureError { .. } => unreachable!(),
        RuntimeError::InvalidWord(span, x) => {
            throw(
                "runtime error",
                &format!("`{x}` is not defined."),
                span.clone(),
                Some("maybe a typo?"),
                call,
            );
        }
        RuntimeError::EmptyDefinition(span, x) => {
            throw(
                "runtime error",
                &format!("definition `{x}` has no value to be associated with. `x` can't be bound to anything."),
                span.clone(),
                Some("add values to the definition body."),
                call,
            );
        }
        RuntimeError::StackUnderflow(span, n, x) => {
            throw(
                "runtime error",
                &format!("operation `{n}` expects {x} element(s) on top of the stack but got a different amount."),
                span.clone(),
                Some("try checking the values before the operation."),
                call,
            );
        }
        RuntimeError::UnexpectedType(span, n, x, y) => {
            throw(
                "runtime error",
                &format!(
                    "operation `{n}` expects {x} datatype(s) on the stack to work, but got {y}."
                ),
                span.clone(),
                Some("try checking the values before the operation."),
                call,
            );
        }
        RuntimeError::ProcRedefinition(span, x) => {
            throw(
                "runtime error",
                &format!("tried to redefine the procedure `{x}` (this name is already taken)."),
                span.clone(),
                None,
                call,
            );
        }
        RuntimeError::DefRedefinition(span, x) => {
            throw(
                "runtime error",
                &format!("tried to redefine the definition `{x}` (this name is already taken)."),
                span.clone(),
                None,
                call,
            );
        }
    }
}

pub fn runtime_error(e: RuntimeError) {
    match e {
        RuntimeError::ProcedureError { call: c, inner: i } => {
            match_runtime_error(i.as_ref(), Some(c));
        }
        x => match_runtime_error(&x, None),
    }
}

pub fn parse_error(e: ParseError) {
    match e {
        ParseError::UnmatchedBlock(span) => {
            throw(
                "parse error",
                "syntax error: found unmatched block: termination of block (end) provided without a beginning (`if`, `else`, `proc`, `def`, or `loop`)",
                span,
                None,
                None,
            );
        }
        ParseError::UnterminatedBlock(span, x) => {
            throw(
                "parse error",
                &format!("syntax error: found unterminated block: termination of block not provided from `{x}` block"),
                span,
                Some("perhaps you forgot to write `end`?"),
                None,
            );
        }
        ParseError::UnexpectedEOF(span, x) => {
            throw(
                "parse error",
                &format!(
                    "syntax error: unexpected end of file while parsing: expected {x} but got the end of the file (nothing)"
                ),
                span,
                None,
                None,
            );
        }
        ParseError::UnexpectedToken(span, x, y) => {
            throw(
                "parse error",
                &format!("syntax error: unexpected token while parsing: expected {y} but got {x}"),
                span,
                None,
                None,
            );
        }
    };
}

pub fn cli_error(e: CLIError) {
    show_usage();
    show_help();
    match e {
        CLIError::ExpectedArgument(x) => {
            fatal(&format!("expected argument: {x}"));
        }
        CLIError::UnexpectedArgument(x) => {
            fatal(&format!("found unexpected argument: {x}"));
        }
    }
}

pub fn fatal(message: &str) {
    eprintln!("pile: fatal: {message}");
    std::process::exit(1);
}

pub fn throw(
    error: &str,
    message: &str,
    span: TokenSpan,
    help: Option<&str>,
    call: Option<TokenSpan>,
) {
    eprintln!(
        "pile: error at {}:{}:{}:",
        span.filename, span.line, span.col
    );
    if let Some(c) = call {
        eprintln!(
            "    > from procedure call at {}:{}:{}:",
            c.filename, c.line, c.col
        );
    }
    eprintln!("    |    {error}:");
    for line in break_line_at(message.to_string(), 50) {
        eprintln!("    |        {line}");
    }
    if let Some(h) = help {
        for line in break_line_at(h.to_string(), 50) {
            eprintln!("    +    {line}");
        }
    }
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
