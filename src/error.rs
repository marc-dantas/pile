use crate::{
    cli::*,
    lexer::TokenSpan,
    parser::ParseError,
    runtime::RuntimeError,
    CLIError,
};
use std::fs::read_to_string;

fn match_runtime_error(e: &RuntimeError, call: Option<TokenSpan>) {
    match e {
        // TODO: plz implement a call stack. this is totally a hack and will not produce good error messages
        e@RuntimeError::ProcedureError { .. } => runtime_error(e.clone()),
        RuntimeError::InvalidWord(span, x) => {
            throw(
                "runtime error",
                &format!("invalid word: `{x}` is not defined."),
                span.clone(),
                Some("maybe a typo?"),
                call,
            );
        }
        RuntimeError::EmptyDefinition(span, x) => {
            throw(
                "runtime error",
                &format!("empty definition: `{x}`."),
                span.clone(),
                Some("add values to the definition body."),
                call,
            );
        }
        RuntimeError::StackUnderflow(span, n, _) => {
            throw(
                "runtime error",
                &format!("stack underflow: not enough values on top of the stack to satisfy `{n}`."),
                span.clone(),
                Some("try checking the values before the operation."),
                call,
            );
        }
        RuntimeError::UnexpectedType(span, n, x, y) => {
            throw(
                "runtime error",
                &format!(
                    "unexpected type: `{n}` expects {x} datatype(s) on the stack to work, but got {y}."
                ),
                span.clone(),
                Some("try checking the values before the operation."),
                call,
            );
        }
        RuntimeError::ProcRedefinition(span, x) => {
            throw(
                "runtime error",
                &format!("procedure redefinition: `{x}`."),
                span.clone(),
                None,
                call,
            );
        }
        RuntimeError::DefRedefinition(span, x) => {
            throw(
                "runtime error",
                &format!("definition redefinition: `{x}`."),
                span.clone(),
                None,
                call,
            );
        }
        RuntimeError::ValueError(span, n, x, y) => {
            throw(
                "runtime error",
                &format!("value error: operation `{n}` expected valid literal value for {x}, but got {y}."),
                span.clone(),
                Some(&format!("likely caused by an invalid conversion to {x}.")),
                call,
            );
        }
        RuntimeError::UnboundVariable(span, s) => {
            throw(
                "runtime error",
                &format!("unbound variable: variable `{s}` has no value to be bound."),
                span.clone(),
                Some(&format!("push values on the stack before using `{s}`.")),
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
                "unmatched block: termination of block (`end`) provided without a beginning.",
                span,
                None,
                None,
            );
        }
        ParseError::UnterminatedBlock(span, x) => {
            throw(
                "parse error",
                &format!("unterminated block: termination of block not provided from `{x}` block."),
                span,
                Some("perhaps you forgot to write `end`?"),
                None,
            );
        }
        ParseError::UnexpectedEOF(span, x) => {
            throw(
                "parse error",
                &format!(
                    "unexpected end of file: expected {x} but got the end of the file (nothing)."
                ),
                span,
                None,
                None,
            );
        }
        ParseError::UnexpectedToken(span, x, y) => {
            throw(
                "parse error",
                &format!("unexpected token: expected {y} but got {x}."),
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

pub fn fatal(message: &str) {
    eprintln!("{GRAY}pile{RES}: {RED}fatal{RES}: {RED}{message}{RES}");
    std::process::exit(1);
}

pub fn throw(
    error: &str,
    message: &str,
    span: TokenSpan,
    help: Option<&str>,
    call: Option<TokenSpan>,
) {
    let mut span = span;
    eprintln!("{GRAY}pile{RES}: {RED}{}{RES}:", error);
    if let Some(c) = call {
        eprintln!("    {GRAY}at {BLUE}{}:{}{RES}:", c.filename, c.line);
        eprintln!(
            "    {GRAY}inside procedure at {BLUE}{}:{}:{}{RES}:\n",
            span.filename, span.line, span.col
        );
        span = c;
    } else {
        eprintln!("    {GRAY}at {BLUE}{}:{}{RES}:\n", span.filename, span.line);
    }
    for (i, j) in read_to_string(span.filename).unwrap().lines().enumerate() {
        if i+1 == span.line {
            eprintln!("    {UNDERWHITE}{}{RES}", j);
            break;
        }
    }
    eprintln!("    {}", " ".repeat(span.col-1) + RED + "^" + RES);
    for line in break_line_at(message.to_string(), 50) {
        eprintln!("    {}{RED}{line}{RES}", " ".repeat(span.col-1));
    }
    if let Some(h) = help {
        for line in break_line_at(h.to_string(), 50) {
            if span.col == 1 {
                eprintln!("  {GREEN}+ {line}{RES}");
            } else if span.col == 2 {
                eprintln!("   {GREEN}+ {line}{RES}");
            } else {
                eprintln!("    {}{GREEN}+ {line}{RES}", " ".repeat(span.col-3));
            }
                
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
