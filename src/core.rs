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

pub fn disassemble_program(program: ProgramTree, filename: &str, import_search_path: Vec<String>) {
    let c = Compiler::new(import_search_path);
    let (instructions, spans) = c.compile(program, filename.to_string());
    println!("{}", filename);
    println!("  {:>18} | instruction", "address");
    for (i, instr) in instructions.iter().enumerate() {
        if let &Instr::SetSpan(s) = instr {
            println!("  0x{:0>16X} | {} ; {}", i, instr, spans.get(s).unwrap());
        } else {
            println!("  0x{:0>16X} | {}", i, instr);
        }
    }
}

pub fn compile_program(program: ProgramTree, filename: String, import_search_path: Vec<String>) -> (Vec<Instr>, Vec<FileSpan>) {
    let c = Compiler::new(import_search_path);
    c.compile(program, filename)
}

pub fn run_program(program: ProgramTree, filename: &str, import_search_path: Vec<String>) -> Result<(), RuntimeError> {
    let (instructions, spans) = compile_program(program, filename.to_string(), import_search_path);
    let r = Executor::new(instructions, spans);
    r.run()
}

pub fn try_run(filename: &str, source: String, import_search_path: Vec<String>) {
    match parse_program(&filename, source) {
        Ok(p) => {
            if let Err(e) = run_program(p, filename, import_search_path) {
                error::runtime_error(e);
            }
        }
        Err(e) => error::parse_error(e),
    }
}
