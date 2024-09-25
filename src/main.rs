mod error;
mod lexer;
mod parser;
mod runtime;
use lexer::*;
use parser::*;
use runtime::*;
use std::env::{args, Args};
use std::fs::read_to_string;

fn read_from_path(filename: &str) -> std::io::Result<String> {
    read_to_string(filename) // this will be improved to something fancier, don't worry
}

fn parse(filename: String, source: String) -> Result<ProgramTree, ParseError> {
    let f = InputFile {
        name: filename,
        content: source.chars().peekable(),
    };
    let l = Lexer::new(f, Span { line: 1, col: 1 });
    let mut p = Parser::new(l);
    p.parse()
}

fn run(p: ProgramTree) -> Result<(), RuntimeError> {
    todo!()
}

fn from_command_line(argv: &mut Args) {
    let program = argv.next().unwrap();
    if let Some(name) = argv.next() {
        if let Ok(source) = read_from_path(&name) {
            match parse(name.to_string(), source) {
                Ok(p) => if let Err(e) = run(p) {
                    error::fatal("a {e}");
                },
                Err(e) => error::parse_error(e),
            }
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
