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

const VERSION: &str = env!("CARGO_PKG_VERSION");

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
        Err(_) => None,
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
            if a.show_help {
                show_usage();
                show_help();
                std::process::exit(0);
            }

            if a.show_version {
                show_version(VERSION);
                std::process::exit(0);
            }

            if let Some(source) = read_file(&a.filename) {
                if a.parse_only {
                    match parse(&a.filename, source) {
                        Ok(p) => println!("{:#?}", p),
                        Err(e) => error::parse_error(e),
                    }
                    std::process::exit(0);
                }
                run(&a.filename, source);
            } else {
                show_usage();
                error::fatal(&format!("couldn't read file {}.", a.filename));
            }
        }
        Err(e) => {
            error::cli_error(e);
        }
    }
}
