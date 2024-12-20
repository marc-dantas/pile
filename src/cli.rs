use std::env::args;
use rustc_version::version;

pub enum CLIError {
    ExpectedArgument(String),
    UnexpectedArgument(String),
}

pub struct Arguments {
    pub filename: String,
    pub show_help: bool,
    pub show_version: bool,
}

impl Arguments {
    fn new(filename: String, show_help: bool, show_version: bool) -> Self {
        Self { filename, show_help, show_version }
    }
}

pub fn show_usage() {
    eprintln!("pile: usage: pile FILENAME [-h] [-v]");
}

pub fn show_help() {
    println!("pile help:");
    println!("  positional arguments:");
    println!("    FILENAME         File path of Pile code");
    println!("  flags:");
    println!("    -h, --help       Show this help message and exit");
    println!("    -v, --version    Show the version information and exit");
}

fn rustc_version() -> String {
    version().unwrap().to_string()
}

pub fn show_version(v: &str) {
    println!("pile programming language {} [RUSTC {}]", v, rustc_version());
}

pub fn parse_arguments() -> Result<Arguments, CLIError> {
    let args = args().skip(1);
    let mut filename = None;
    let mut show_help = false;
    let mut show_version = false;

    for arg in args.into_iter() {
        match arg.as_str() {
            "-h" | "--help" => show_help = true,
            "-v" | "--version" => show_version = true,
            _ => {
                if let Some(_) = filename {
                    return Err(CLIError::UnexpectedArgument(arg));
                }
                filename = Some(arg);
            }
        }
    }
    
    if let Some(f) = filename {
        Ok(Arguments::new(f, show_help, show_version))
    } else {
        if show_help || show_version {
            return Ok(Arguments::new("".to_string(), show_help, show_version));
        }
        Err(CLIError::ExpectedArgument("FILENAME".to_string()))
    }
}