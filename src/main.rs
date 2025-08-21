mod cli;
mod error;
mod lexer;
mod compiler;
mod parser;
mod runtime;
mod util;
use util::*;
use std::fs::File;
use std::io::Read;
use cli::*;

const VERSION: &str = env!("CARGO_PKG_VERSION");

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

            let source = try_read_file(&a.filename);
            
            if a.disassemble {
                disassemble_program(
                    try_parse(&a.filename, source),
                    &a.filename,
                );
                std::process::exit(0);
            }

            if a.parse_only {
                println!("{:#?}", try_parse(&a.filename, source));
                std::process::exit(0);
            }
            try_run(&a.filename, source);
        }
        Err(e) => {
            error::cli_error(e);
        }
    }
}
