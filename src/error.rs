use crate::{
    cli::*,
    lexer::FileSpan,
    parser::ParseError,
    runtime::RuntimeError,
    CLIError,
};

fn match_runtime_error(e: &RuntimeError, call: Option<FileSpan>) {
    match e {
        // // TODO: plz implement a call stack. this is totally a hack and will not produce good error messages
        // RuntimeError::ProcedureError { inner, call: proc_call } => {
        //     if let Some(c) = call {
        //         match_runtime_error(inner, Some(c));
        //     } else {
        //         match_runtime_error(inner, Some(proc_call.clone()));
        //     }
        // }
        // RuntimeError::RecursionDepthOverflow(span) => {
        //     throw(
        //         "runtime error",
        //         &format!("recursion depth overflow: procedure call lead to too many recursive iterations."),
        //         span.clone(),
        //         Some("probably you are doing something wrong."),
        //         call,
        //     );
        // }
        // RuntimeError::ImportError(span, path) => {
        //     throw(
        //         "runtime error",
        //         &format!("import error: could not import \"{path}\"."),
        //         span.clone(),
        //         Some("maybe the file you are trying to import doesn't actually exist."),
        //         call,
        //     );
        // }
        // RuntimeError::ReadMemoryOutOfBounds(span, addr) => {
        //     throw(
        //         "runtime error",
        //         &format!("memory out of bounds: tried to read memory at invalid address 0X{:X} ({addr}).", addr),
        //         span.clone(),
        //         Some("check if you are using `mem` correctly."),
        //         call,
        //     );
        // }
        // RuntimeError::WriteMemoryOutOfBounds(span, what, addr) => {
        //     throw(
        //         "runtime error",
        //         &format!("memory out of bounds: tried to write {what} memory at invalid address 0X{:X} ({addr}).", addr),
        //         span.clone(),
        //         Some("check if you are using `mem` correctly."),
        //         call,
        //     );
        // }
        RuntimeError::ArrayOutOfBounds(span, index, len) => {
            throw(
                "runtime error",
                &format!("array index out of bounds: tried to index array of size {len} but used index {index}."),
                span.clone(),
                None,
                call,
            );
        }
        RuntimeError::StringOutOfBounds(span, index, len) => {
            throw(
                "runtime error",
                &format!("string index out of bounds: tried to index string of size {len} but used index {index}."),
                span.clone(),
                None,
                call,
            );
        }
        RuntimeError::InvalidSymbol(span, x) => {
            throw(
                "runtime error",
                &format!("invalid symbol: `{x}` is not defined."),
                span.clone(),
                Some("maybe a typo?"),
                call,
            );
        }
        RuntimeError::EmptyDefinition(span, x) => {
            throw(
                "runtime error",
                &format!("found empty definition: the expression inside {x} leads to no value on the stack."),
                span.clone(),
                None,
                call,
            );
        }
        RuntimeError::StackUnderflow(span, op, n) => {
            throw(
                "runtime error",
                &format!("stack underflow: too few values on the stack to satisfy `{op}` (expected {n})"),
                span.clone(),
                Some(&format!("use `trace` operation to see the values on the stack without removing them.")),
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
        RuntimeError::DivisionByZero(span) => {
            throw(
                "runtime error",
                &format!("division by zero."),
                span.clone(),
                None,
                call,
            );
        }
        // RuntimeError::ValueError(span, n, x, y) => {
        //     throw(
        //         "runtime error",
        //         &format!("value error: operation `{n}` expected valid literal value for {x}, but got {y}."),
        //         span.clone(),
        //         Some(&format!("likely caused by an invalid conversion to {x}.")),
        //         call,
        //     );
        // }
        // RuntimeError::UnboundVariable(span, s) => {
        //     throw(
        //         "runtime error",
        //         &format!("unbound variable: variable `{s}` has no value to be bound."),
        //         span.clone(),
        //         Some(&format!("push values on the stack before using `{s}`.")),
        //         call,
        //     );
        // }
    }
}

pub fn runtime_error(e: RuntimeError) {
    match e {
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
    eprintln!("pile: fatal: {message}");
    std::process::exit(1);
}

pub fn throw(
    error: &str,
    message: &str,
    span: FileSpan,
    help: Option<&str>,
    call: Option<FileSpan>,
) {
    eprintln!("pile: {}:", error);
    if let Some(s) = call {
        eprintln!(" |  {}:{}:{}:", s.filename, s.line, s.col);
        eprintln!(" |    {}:{}:{}:", span.filename, span.line, span.col);
    } else {
        eprintln!(" |  {}:{}:{}:", span.filename, span.line, span.col);
    }
    for line in break_line_at(message.to_string(), 50) {
        eprintln!(" |      {line}",);
    }
    if let Some(h) = help {
        for line in break_line_at(h.to_string(), 50) {
            eprintln!(" +  {}", line);
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
