use rustc_version::version_meta;
use std::env::args;

pub enum CLIError {
    InvalidFlag(String),
    ExpectedArgument(String),
    UnexpectedArgument(String),
}

pub struct Arguments {
    pub filename: String,
    pub show_help: bool,
    pub disassemble: bool,
    pub show_version: bool,
    pub parse_only: bool,
}


pub fn show_usage() {
    eprintln!("\x1B[1;32musage\x1B[0m: pile <program.pile> [OPTIONS]");
}

pub fn show_help() {
    println!("\x1B[1;32mhelp\x1B[0m:");
    println!("  positional arguments:");
    println!("    <program.pile>        \tFile path to Pile source-code");
    println!("  optional arguments:");
    println!("    -h | --help           \tShow this help message and exit");
    println!("    -v | --version        \tShow the version information and exit");
    println!("    -P | --parse-only     \tParse FILENAME and write parser result to stdout");
    println!("    -I | --import <PATH>  \tAdd PATH as an option to import search paths");
    println!("    -D | --disassemble    \tDisassemble the compiled program and write to stdout");
}

fn rustc_version() -> String {
    let v = version_meta().unwrap();
    return format!("{} {}", v.short_version_string, v.host);
}

pub fn show_version(v: &str) {
    println!("pile programming language {}", v);
    println!("{}", rustc_version());
}

pub fn parse_arguments() -> Result<Arguments, CLIError> {
    let args = args().skip(1);
    let mut filename = None;
    let mut show_help = false;
    let mut show_version = false;
    let mut parse_only = false;
    let mut disassemble = false;

    for arg in args.into_iter() {
        match arg.as_str() {
            flag if arg.starts_with("-") => match flag {
                "-h" | "--help" => show_help = true,
                "-v" | "--version" => show_version = true,
                "-P" | "--parse-only" => parse_only = true,
                "-D" | "--disassemble" => disassemble = true,
                _ => return Err(CLIError::InvalidFlag(flag.to_string())),
            },
            _ => {
                if let Some(_) = filename {
                    return Err(CLIError::UnexpectedArgument(arg));
                }
                filename = Some(arg);
            }
        }
    }

    if let Some(f) = filename {
        Ok(Arguments::new(f, show_help, show_version, disassemble, parse_only))
    } else {
        if show_help || show_version {
            return Ok(Arguments::new(
                "".to_string(),
                show_help,
                disassemble,
                show_version,
                parse_only,
            ));
        }
        Err(CLIError::ExpectedArgument("FILENAME".to_string()))
    }
}
