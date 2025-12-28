mod cli;
mod error;
mod lexer;
mod compiler;
mod parser;
mod runtime;
mod core;

use std::env;

use core::*;
use cli::*;

const VERSION: &str = env!("CARGO_PKG_VERSION");

const PILE_IMPORT_SEARCH_PATHS: &[&'static str] = &["$HOME/pile/", "%UserProfile%\\pile\\", "./"];

fn main() {
    let mut search_paths = PILE_IMPORT_SEARCH_PATHS.iter().map(|x| String::from(*x)).collect::<Vec<String>>();
    match parse_arguments() {
        Ok(a) => {
            search_paths.extend(a.search_paths);

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
                    search_paths
                );
                std::process::exit(0);
            }
            if a.parse_only {
                println!("{:#?}", try_parse(&a.filename, source));
                std::process::exit(0);
            }
            try_run(&a.filename, source, search_paths);
        }
        Err(e) => {
            error::cli_error(e);
        }
    }
}
