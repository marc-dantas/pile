mod cli;
mod error;
mod lexer;
mod parser;
mod runtime;
use cli::*;
use lexer::*;
use parser::*;
use runtime::*;
use std::fs::File;
use std::io::Read;

fn read_file(path: &str) -> Option<String> {
    match File::open(path) {
        Ok(mut f) => {
            let mut xs = Vec::new();
            f.read_to_end(&mut xs).unwrap();
            match String::from_utf8(xs) {
                Ok(x) => return Some(x),
                Err(_) => return None,
            }
        }
        Err(_) => None
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

fn main() {
    match parse_arguments() {
        Ok(a) => {
            if let Some(source) = read_file(&a.filename) {
                run(&a.filename, source);
            } else {
                error::usage("pile");
                error::fatal(&format!("couldn't read file {}.", a.filename));
            }
        }
        Err(e) => {
            error::cli_error(e);
        }
    }
}
