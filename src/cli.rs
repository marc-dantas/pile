use std::env::args;

pub enum CLIError {
    UnexpectedFlag(String),
    ExpectedArgument(String),
}

pub struct Arguments {
    pub filename: String,
    // Flags can be added here
}

impl Arguments {
    fn new(filename: String) -> Self {
        Self { filename }
    }
}

pub fn parse_arguments() -> Result<Arguments, CLIError> {
    let mut args = args();
    args.next();

    while let Some(arg) = args.next() {
        if arg.starts_with('-') {
            return Err(CLIError::UnexpectedFlag(arg));
        } else {
            return Ok(Arguments::new(arg));
        }
    }

    // If no filename was provided
    Err(CLIError::ExpectedArgument("FILENAME".to_string()))
}
