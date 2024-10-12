use crate::{parser::{ProgramTree, OpKind, Node}, lexer::TokenSpan};
use std::{clone, collections::VecDeque};

#[derive(Debug)]
pub enum Data {
    String(String),
    Number(f64),
}

#[derive(Debug)]
pub struct Procedure<'a>(String, &'a Vec<Node>);

#[derive(Debug)]
pub struct Definition(String, Data);

#[derive(Debug)]
pub struct Namespace<'a> {
    pub procs: Vec<Procedure<'a>>,
    pub defs: Vec<Definition>,
}

pub enum NumberBinaryOp {
    Add, Sub, Mul, Div,
    Gt, Lt, Eq, Ge, Le, Ne,
    Shl, Shr, Bor, Band,
    Swap, Over
}

// stack operations:
// dup (a -- a a)
// drop (a -- )
// swap (a b -- b a)
// over (a b -- a b a)
// rot (a b c -- b c a)

pub enum NumberUnaryOp {
    Print, Dup, Drop
}

pub fn numberunaryop_readable(x: NumberUnaryOp) -> String {
    return match x {
        NumberUnaryOp::Print => String::from("print"),
        NumberUnaryOp::Dup => String::from("dup"),
        NumberUnaryOp::Drop => String::from("drop"),
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
        NumberBinaryOp::Band => String::from("&"),
        NumberBinaryOp::Swap => String::from("swap"),
        NumberBinaryOp::Over => String::from("over"),
    };
}


pub type Stack = VecDeque<Data>;

#[derive(Debug)]
pub enum RuntimeError {
    StackUnderflow(TokenSpan, String, usize),          // when there's too few data on the stack to perform operation
    StackOverflow(TokenSpan, usize),                   // when there's too much data on the stack (leftover unhandled data)
    UnexpectedType(TokenSpan, String, String, String), // when there's an operation tries to operate with an unsupported or an invalid datatype
    InvalidOp(TokenSpan, String),                      // used when a word doesn't correspond a valid operation
    InvalidWord(TokenSpan, String),                    // used when a word doesn't correspond a valid identifier
    ProcRedefinition(TokenSpan, String),               // used when a name is already taken
    EmptyDefinition(TokenSpan, String),                // used when a definition has empty body
}

pub struct Runtime<'a> {
    input: &'a ProgramTree,
    stack: Stack,
    namespace: Namespace<'a>,
}

impl<'a> Runtime<'a> {
    pub fn new(input: &'a ProgramTree) -> Self {
        Self {
            input,
            stack: VecDeque::new(),
            namespace: Namespace {
                procs: Vec::new(),
                defs: Vec::new(),
            }
        }
    }

    fn unop_number(&mut self, span: TokenSpan, x: NumberUnaryOp) -> Result<(), RuntimeError> {
        if let Some(a) = self.pop() {
            match a {
                Data::Number(n) => match x {
                    NumberUnaryOp::Print => println!("{}", n),
                    NumberUnaryOp::Dup => self.push_number(n),
                    NumberUnaryOp::Drop => self.drop(),
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
                (Data::Number(n1), Data::Number(n2)) => match x {
                    NumberBinaryOp::Add => self.push_number(n1 + n2),
                    NumberBinaryOp::Sub => self.push_number(n1 - n2),
                    NumberBinaryOp::Mul => self.push_number(n1 * n2),
                    NumberBinaryOp::Div => self.push_number(n1 / n2),
                    NumberBinaryOp::Eq => self.push_number((n1 == n2) as i32 as f64),
                    NumberBinaryOp::Ne => self.push_number((n1 != n2) as i32 as f64),
                    NumberBinaryOp::Lt => self.push_number((n1 < n2) as i32 as f64),
                    NumberBinaryOp::Gt => self.push_number((n1 > n2) as i32 as f64),
                    NumberBinaryOp::Le => self.push_number((n1 <= n2) as i32 as f64),
                    NumberBinaryOp::Ge => self.push_number((n1 >= n2) as i32 as f64),
                    NumberBinaryOp::Shl => self.push_number(((n1 as i32) << (n2 as i32)) as f64),
                    NumberBinaryOp::Shr => self.push_number(((n1 as i32) >> (n2 as i32)) as f64),
                    NumberBinaryOp::Bor => self.push_number(((n1 as i32) | (n2 as i32)) as f64),
                    NumberBinaryOp::Band => self.push_number(((n1 as i32) & (n2 as i32)) as f64),
                    NumberBinaryOp::Swap => {
                        self.push_number(n1);
                        self.push_number(n2);
                    },
                    NumberBinaryOp::Over => {
                        self.push_number(n2);
                        self.push_number(n1);
                        self.push_number(n2);
                    },
                },
                (Data::String(_), Data::String(_)) => return Err(RuntimeError::UnexpectedType(span, numberbinaryop_readable(x), "(number, number)".to_string(), "(string, string)".to_string())),
                (Data::String(_), Data::Number(_)) => return Err(RuntimeError::UnexpectedType(span, numberbinaryop_readable(x), "(number, number)".to_string(), "(string, number)".to_string())),
                (Data::Number(_), Data::String(_)) => return Err(RuntimeError::UnexpectedType(span, numberbinaryop_readable(x), "(number, number)".to_string(), "(number, string)".to_string())),
            }
        } else {
            return Err(RuntimeError::StackUnderflow(span, numberbinaryop_readable(x), 2));
        }
        Ok(())
    }

    fn run_node(&mut self, n: &'a Node) -> Result<(), RuntimeError> {
        match n {
            Node::If(i, e, s) => {},
            Node::Loop(l, s) => {},
            Node::Proc(n, p, s) => {
                if let Some(_) = self.namespace.procs.iter().find(|p| p.0 == *n) {
                    return Err(RuntimeError::ProcRedefinition(s.clone(), n.to_string()));
                }
                self.namespace.procs.push(Procedure(n.to_string(), p));
            },
            Node::Def(n, p, s) => {
                if let Some(_) = self.namespace.defs.iter().find(|p| p.0 == *n) {
                    return Err(RuntimeError::ProcRedefinition(s.clone(), n.to_string()));
                }
                // TODO: find a way to isolate the stack from the main stack
                // so that the definition can be executed without affecting the main stack
                // this is needed because the definition can be executed multiple times
                // and we don't want to affect the main stack
                self.run_block(p)?;
                if let Some(result) = self.pop() {
                    self.namespace.defs.push(Definition(n.to_string(), result));
                } else {
                    return Err(RuntimeError::EmptyDefinition(s.clone(), n.to_string()));
                }
            },
            Node::Number(n, s) => self.push_number(*n),
            Node::String(v, s) => self.push_string(v.to_string()),
            Node::Operation(op, s) => {
                let s = s.clone(); // TODO: this is a hack, fix it
                match op {
                    OpKind::Add => self.binop_number(s, NumberBinaryOp::Add)?,
                    OpKind::Sub => self.binop_number(s, NumberBinaryOp::Sub)?,
                    OpKind::Mul => self.binop_number(s, NumberBinaryOp::Mul)?,
                    OpKind::Div => self.binop_number(s, NumberBinaryOp::Div)?,
                    OpKind::Gt => self.binop_number(s, NumberBinaryOp::Gt)?,
                    OpKind::Lt => self.binop_number(s, NumberBinaryOp::Lt)?,
                    OpKind::Eq => self.binop_number(s, NumberBinaryOp::Eq)?,
                    OpKind::Ge => self.binop_number(s, NumberBinaryOp::Ge)?,
                    OpKind::Le => self.binop_number(s, NumberBinaryOp::Le)?,
                    OpKind::Ne => self.binop_number(s, NumberBinaryOp::Ne)?,
                    OpKind::Shl => self.binop_number(s, NumberBinaryOp::Shl)?,
                    OpKind::Shr => self.binop_number(s, NumberBinaryOp::Shr)?,
                    OpKind::Bor => self.binop_number(s, NumberBinaryOp::Bor)?,
                    OpKind::Band => self.binop_number(s, NumberBinaryOp::Band)?,
                    OpKind::Swap => self.binop_number(s, NumberBinaryOp::Swap)?,
                    OpKind::Over => self.binop_number(s, NumberBinaryOp::Over)?,
                    OpKind::Dup => self.unop_number(s, NumberUnaryOp::Dup)?,
                    OpKind::Drop => self.unop_number(s, NumberUnaryOp::Drop)?,
                    OpKind::Print => self.unop_number(s, NumberUnaryOp::Print)?,
                    OpKind::Rot => { todo!() },
                }
            },
            Node::Word(w, s) => {
                if let Some(p) = self.namespace.procs.iter().find(|p| p.0 == *w) {
                    self.run_block(&p.1)?;
                } else if let Some(d) = self.namespace.defs.iter().find(|p| p.0 == *w) {
                    match &d.1 {
                        Data::Number(n) => self.push_number(*n),
                        Data::String(s) => self.push_string(String::from(s)),
                    }
                } else {
                    return Err(RuntimeError::InvalidWord(s.clone(), w.to_string()));
                }
            }
        }
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), RuntimeError> {
        for n in self.input {
            self.run_node(n)?;
        }
        Ok(())
    }

    fn run_block(&mut self, b: &'a Vec<Node>) -> Result<(), RuntimeError> {
        for n in b {
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

    fn push_data(&mut self, d: Data) {
        self.stack.push_front(d);
    }

    fn pop(&mut self) -> Option<Data> {
        self.stack.pop_front()
    }

    fn drop(&mut self) {
        self.stack.remove(0);
    }
}
