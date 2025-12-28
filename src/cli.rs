use rustc_version::version_meta;
use std::env::args;

pub enum CLIError {
    InvalidFlag(String),
    ExpectedArgument(String),
    UnexpectedArgument(String),
}

pub struct Arguments {
    pub filename: String,
    pub search_paths: Vec<String>,
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
    let args = args().skip(1).collect::<Vec<String>>();
    let mut filename = None;
    let mut search_paths = Vec::new();
    let mut show_help = false;
    let mut show_version = false;
    let mut parse_only = false;
    let mut disassemble = false;

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        match arg.as_str() {
            flag if arg.starts_with("-") => match flag {
                "-h" | "--help" => show_help = true,
                "-v" | "--version" => show_version = true,
                "-P" | "--parse-only" => parse_only = true,
                "-D" | "--disassemble" => disassemble = true,
                "-I" | "--import" => {
                    if i+1 >= args.len() {
                        return Err(CLIError::ExpectedArgument(format!("for {flag} flag")))
                    }
                    let next = &args[i + 1];
                    search_paths.push(next.clone());
                    i += 1;
                },
                _ => return Err(CLIError::InvalidFlag(flag.to_string())),
            },
            _ => {
                if let Some(_) = filename {
                    return Err(CLIError::UnexpectedArgument(arg.clone()));
                }
                filename = Some(arg.clone());
            }
        }
        i += 1;
    }

    if let Some(f) = filename {
        return Ok(Arguments {
            filename: f,
            search_paths,
            show_help,
            show_version,
            disassemble,
            parse_only
        });
    } else if show_help || show_version {
        return Ok(Arguments {
            filename: "".to_string(),
            search_paths,
            show_help,
            disassemble,
            show_version,
            parse_only,
        });
    }
    Err(CLIError::ExpectedArgument("FILENAME".to_string()))
}
