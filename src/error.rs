use crate::parser::ParseError;

pub fn parse_error(e: ParseError) {
    match e {
        ParseError::UnmatchedBlock(span) => {
            throw(
                "parse error",
                "syntax error: found unmatched block: termination of block (end) provided without a beginning (`if`, `else`, `proc`, or `loop`)",
                &span.filename,
                span.line,
                span.col,
                None
            );
        }
        ParseError::UnterminatedBlock(span, x) => {
            throw(
                "parse error",
                &format!("syntax error: found unterminated block: termination of block not provided from `{x}` block"),
                &span.filename,
                span.line,
                span.col,
                Some("perhaps you forgot to write `end`?")
            );
        }
        ParseError::UnexpectedEOF(span, x) => {
            throw(
                "parse error",
                &format!(
                    "syntax error: unexpected end of file while parsing: expected {x} but got the end of the file (nothing)"
                ),
                &span.filename,
                span.line,
                span.col,
                None,
            );
        }
        ParseError::UnexpectedToken(span, x, y) => {
            throw(
                "parse error",
                &format!("syntax error: unexpected token while parsing: expected {y} but got {x}"),
                &span.filename,
                span.line,
                span.col,
                None,
            );
        }
    };
}

pub fn usage(program: &str) {
    eprintln!("pile: usage: {program} FILENAME");
}

pub fn fatal(message: &str) {
    eprintln!("pile: fatal: {message}");
    std::process::exit(1);
}

pub fn warning(message: &str) {
    eprintln!("pile: warning: {message}");
}

pub fn throw(
    error: &str,
    message: &str,
    file: &String,
    line: usize,
    col: usize,
    help: Option<&str>,
) {
    eprintln!("pile: error at {file}:{line}:{col}:");
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
