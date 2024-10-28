mod error;
mod lexer;
mod parser;
mod runtime;
use lexer::*;
use parser::*;
use runtime::*;
use std::env::{args, Args};
use std::fs::File;
use std::io::Read;

fn read_file(path: &str) -> Option<String> {
    let mut file = File::open(path).unwrap();
    let mut xs = Vec::new();
    file.read_to_end(&mut xs).unwrap();
    match String::from_utf8(xs) {
        Ok(x) => Some(x),
        Err(_) => {
            None
        }
    }
}

fn parse(filename: &str, source: String) -> Result<ProgramTree, ParseError> {
    let f = InputFile {
        name: filename,
        content: source.chars().peekable(),
    };
    let l = Lexer::new(f, Span { line: 1, col: 1 });
    let mut p = Parser::new(l);
    p.parse()
}

fn run_program(p: ProgramTree) -> Result<(), RuntimeError> {
    let mut r = Runtime::new(&p);
    r.run()
}

fn run(filename: &str, source: String) {
    match parse(&filename, source) {
        Ok(p) => {
            if let Err(e) = run_program(p) {
                error::runtime_error(e);
            }
        }
        Err(e) => error::parse_error(e),
    }
}

fn from_command_line(argv: &mut Args) {
    let program = argv.next().unwrap();
    if let Some(name) = argv.next() {
        if let Some(source) = read_file(&name) {
            run(&name, source);
        } else {
            error::usage(&program);
            error::fatal(&format!("couldn't read file {}.", name));
        }
    } else {
        error::usage(&program);
        error::fatal("expected positional argument FILENAME");
    }
}

fn main() {
    let mut argv = args();
    from_command_line(&mut argv);
}
