use crate::{parser::ProgramTree, lexer::TokenSpan};
use std::collections::VecDeque;

pub enum Data {
    String(String),
    Number(f64),
}

pub type Stack = VecDeque<Data>;

pub struct Runtime {
    input: ProgramTree,
    stack: Stack,
}

pub enum RuntimeError {

}

impl Runtime {
    pub fn new(input: ProgramTree) -> Self {
        Self {
            input,
            stack: VecDeque::new()
        }
    }

    pub fn run() -> Result<(), RuntimeError> {
        panic!();
    }

    // ...
}