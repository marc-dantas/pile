use crate::{parser::{ProgramTree, Node}, lexer::TokenSpan};
use std::{collections::VecDeque, ops::Add, str::Bytes};

#[derive(Debug)]
pub enum Data {
    String(String),
    Number(f64),
}

pub enum NumberBinaryOp {
    Add, Sub, Mul, Div,
    Gt, Lt, Eq, Ge, Le, Ne,
    Shl, Shr, Bor, Band
}

pub enum NumberUnaryOp {
    Print,
}

pub fn numberunaryop_readable(x: NumberUnaryOp) -> String {
    return match x {
        NumberUnaryOp::Print => String::from("print"),
    };
}

// maybe implement some Format thing idk
pub fn numberbinaryop_readable(x: NumberBinaryOp) -> String {
    return match x {
        NumberBinaryOp::Add => String::from("+"),
        NumberBinaryOp::Sub => String::from("-"),
        NumberBinaryOp::Mul => String::from("*"),
        NumberBinaryOp::Div => String::from("/"),
        NumberBinaryOp::Gt => String::from(">"),
        NumberBinaryOp::Lt => String::from("<"),
        NumberBinaryOp::Eq => String::from("="),
        NumberBinaryOp::Ge => String::from(">="),
        NumberBinaryOp::Le => String::from("<="),
        NumberBinaryOp::Ne => String::from("!="),
        NumberBinaryOp::Shl => String::from(">>"),
        NumberBinaryOp::Shr => String::from("<<"),
        NumberBinaryOp::Bor => String::from("|"),
        NumberBinaryOp::Band => String::from("&")
    };
}


pub type Stack = VecDeque<Data>;

#[derive(Debug)]
pub enum RuntimeError {
    StackUnderflow(TokenSpan, String, usize),          // when there's too few data on the stack to perform operation
    StackOverflow(TokenSpan, usize),                   // when there's too much data on the stack (leftover unhandled data)
    UnexpectedType(TokenSpan, String, String, String), // when there's an operation tries to operate with an unsupported or an invalid datatype
    InvalidOp(TokenSpan, String),                      // used when a word doesn't correspond a valid operation
    InvalidName(TokenSpan, String),                    // used when a word doesn't correspond a valid identifier
}

pub struct Runtime<'a> {
    input: &'a ProgramTree,
    stack: Stack,
}

impl<'a> Runtime<'a> {
    pub fn new(input: &'a ProgramTree) -> Self {
        Self {
            input,
            stack: VecDeque::new(),
        }
    }

    fn unop_number(&mut self, span: TokenSpan, x: NumberUnaryOp) -> Result<(), RuntimeError> {
        if let Some(a) = self.pop() {
            match a {
                Data::Number(n) => match x {
                    NumberUnaryOp::Print => println!("{}", n),
                },
                Data::String(_) => return Err(RuntimeError::UnexpectedType(span, numberunaryop_readable(x), "(number)".to_string(), "(string)".to_string())),
            }
        } else {
            return Err(RuntimeError::StackUnderflow(span, numberunaryop_readable(x), 1));
        }
        Ok(())
    }

    fn binop_number(&mut self, span: TokenSpan, x: NumberBinaryOp) -> Result<(), RuntimeError> {
        if let (Some(a), Some(b)) = (self.pop(), self.pop()) {
            match (a, b) {
                (Data::Number(n1), Data::Number(n2)) => self.push_number(match x {
                    NumberBinaryOp::Add => n1 + n2,
                    NumberBinaryOp::Sub => n1 - n2,
                    NumberBinaryOp::Mul => n1 * n2,
                    NumberBinaryOp::Div => n1 / n2,
                    NumberBinaryOp::Eq => (n1 == n2) as i32 as f64,
                    NumberBinaryOp::Ne => (n1 != n2) as i32 as f64,
                    NumberBinaryOp::Lt => (n1 < n2) as i32 as f64,
                    NumberBinaryOp::Gt => (n1 > n2) as i32 as f64,
                    NumberBinaryOp::Le => (n1 <= n2) as i32 as f64,
                    NumberBinaryOp::Ge => (n1 >= n2) as i32 as f64,
                    NumberBinaryOp::Shl => ((n1 as i32) << (n2 as i32)) as f64,
                    NumberBinaryOp::Shr => ((n1 as i32) >> (n2 as i32)) as f64,
                    NumberBinaryOp::Bor => ((n1 as i32) | (n2 as i32)) as f64,
                    NumberBinaryOp::Band => ((n1 as i32) & (n2 as i32)) as f64,
                }),
                (Data::String(_), Data::String(_)) => return Err(RuntimeError::UnexpectedType(span, numberbinaryop_readable(x), "(number, number)".to_string(), "(string, string)".to_string())),
                (Data::String(_), Data::Number(_)) => return Err(RuntimeError::UnexpectedType(span, numberbinaryop_readable(x), "(number, number)".to_string(), "(string, number)".to_string())),
                (Data::Number(_), Data::String(_)) => return Err(RuntimeError::UnexpectedType(span, numberbinaryop_readable(x), "(number, number)".to_string(), "(number, string)".to_string())),
            }
        } else {
            return Err(RuntimeError::StackUnderflow(span, numberbinaryop_readable(x), 2));
        }
        Ok(())
    }

    fn run_node(&mut self, n: &Node) -> Result<(), RuntimeError> {
        match n {
            Node::If(i, e, s) => {},
            Node::Loop(l, s) => {},
            Node::Procedure(n, p, s) => {},
            Node::Number(n, s) => self.push_number(*n),
            Node::String(v, s) => self.push_string(v.to_string()),
            Node::Operation(o, s) => {
                let s = s.clone(); // TODO: Solve this
                match o.as_str() {
                    "print" => self.unop_number(s, NumberUnaryOp::Print)?,
                    "+" => self.binop_number(s, NumberBinaryOp::Add)?,
                    "-" => self.binop_number(s, NumberBinaryOp::Sub)?,
                    "*" => self.binop_number(s, NumberBinaryOp::Mul)?,
                    "/" => self.binop_number(s, NumberBinaryOp::Div)?,
                    ">" => self.binop_number(s, NumberBinaryOp::Gt)?,
                    "<" => self.binop_number(s, NumberBinaryOp::Lt)?,
                    "=" => self.binop_number(s, NumberBinaryOp::Eq)?,
                    "<=" => self.binop_number(s, NumberBinaryOp::Le)?,
                    ">=" => self.binop_number(s, NumberBinaryOp::Ge)?,
                    "!=" => self.binop_number(s, NumberBinaryOp::Ne)?,
                    "<<" => self.binop_number(s, NumberBinaryOp::Shl)?,
                    ">>" => self.binop_number(s, NumberBinaryOp::Shr)?,
                    "|" => self.binop_number(s, NumberBinaryOp::Bor)?,
                    "&" => self.binop_number(s, NumberBinaryOp::Band)?,
                    _ => return Err(RuntimeError::InvalidOp(s, o.clone())),
                }
            },
            // Node::Word(w, s) => {}
        }
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), RuntimeError> {
        for n in self.input {
            self.run_node(n)?;
        }
        Ok(())
    }


    fn push_number(&mut self, n: f64) {
        self.stack.push_front(Data::Number(n));
    }

    fn push_string(&mut self, s: String) {
        self.stack.push_front(Data::String(s));
    }

    fn pop(&mut self) -> Option<Data> {
        self.stack.pop_front()
    }

    fn drop(&mut self) -> Option<Data> {
        self.stack.remove(0)
    }
}
