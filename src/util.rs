use crate::lexer;
use crate::compiler;
use crate::parser;
use crate::runtime;
use crate::error;
use compiler::*;
use lexer::*;
use parser::*;
use runtime::*;

pub fn read_file(path: &str) -> Option<String> {
    use std::io::Read;
    use std::fs::File;
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

pub fn try_read_file(path: &str) -> String {
    match read_file(path) {
        Some(s) => return s,
        None => {
            error::fatal(&format!("couldn't read file {}.", path));
        }
    };
}

pub fn parse_program(filename: &str, source: String) -> Result<ProgramTree, ParseError> {
    let f = InputFile {
        name: filename,
        content: source.chars().peekable(),
    };
    let l = Lexer::new(f, Span { line: 1, col: 1 });
    let mut p = Parser::new(l);
    p.parse()
}

pub fn try_parse(filename: &str, source: String) -> ProgramTree {
    match parse_program(filename, source) {
        Ok(p) => return p,
        Err(e) => error::parse_error(e),
    }
    std::process::exit(0);
}

pub fn disassemble_program(program: ProgramTree, filename: &str) {
    let c = Compiler::new();
    let instructions = c.compile(program, filename.to_string());
    println!("{}", filename);
    println!("  {:>18} | instruction", "address");
    for (i, instr) in instructions.iter().enumerate() {
        println!("  0x{:0>16X} | {}", i, instr);
    }
}

pub fn compile_program(program: ProgramTree, filename: String) -> Vec<Instr> {
    let c = Compiler::new();
    c.compile(program, filename)
}

pub fn try_compile_from_file(filename: String) -> Vec<Instr> {
    compile_program(try_parse(&filename, try_read_file(&filename)), filename)
}

pub fn run_program(program: ProgramTree, filename: &str) -> Result<(), RuntimeError> {
    let instructions = compile_program(program, filename.to_string());
    let r = Executor::new(instructions);
    r.run()
}

pub fn try_run(filename: &str, source: String) {
    match parse_program(&filename, source) {
        Ok(p) => {
            if let Err(e) = run_program(p, filename) {
                error::runtime_error(e);
            }
        }
        Err(e) => error::parse_error(e),
    }
}
