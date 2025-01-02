use crate::{
    lexer::TokenSpan,
    parser::{Node, OpKind, ProgramTree},
};
use std::{
    collections::VecDeque,
    io::{Read, Write},
    str::FromStr,
};

#[derive(Debug)]
pub enum Data {
    String(String),
    Number(f64),
}

impl std::fmt::Display for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Data::String(_) => write!(f, "string"),
            Data::Number(_) => write!(f, "number"),
        }
    }
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

// stack operations:
// dup (a -- a a)
// drop (a -- )
// swap (a b -- b a)
// over (a b -- a b a)
// rot (a b c -- b c a)

pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Exp,
    Gt,
    Lt,
    Eq,
    Ge,
    Le,
    Ne,
    Shl,
    Shr,
    Bor,
    Band,
    Swap,
    Over,
}

impl std::fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            BinaryOp::Add => write!(f, "+"),
            BinaryOp::Sub => write!(f, "-"),
            BinaryOp::Mul => write!(f, "*"),
            BinaryOp::Div => write!(f, "/"),
            BinaryOp::Mod => write!(f, "%"),
            BinaryOp::Exp => write!(f, "**"),
            BinaryOp::Gt => write!(f, ">"),
            BinaryOp::Lt => write!(f, "<"),
            BinaryOp::Eq => write!(f, "="),
            BinaryOp::Ge => write!(f, ">="),
            BinaryOp::Le => write!(f, "<="),
            BinaryOp::Ne => write!(f, "!="),
            BinaryOp::Shl => write!(f, ">>"),
            BinaryOp::Shr => write!(f, "<<"),
            BinaryOp::Bor => write!(f, "|"),
            BinaryOp::Band => write!(f, "&"),
            BinaryOp::Swap => write!(f, "swap"),
            BinaryOp::Over => write!(f, "over"),
        }
    }
}

pub enum UnaryOp {
    Trace,
    Dup,
    Drop,
    BNot,
}

impl std::fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            UnaryOp::Trace => write!(f, "trace"),
            UnaryOp::Dup => write!(f, "dup"),
            UnaryOp::Drop => write!(f, "drop"),
            UnaryOp::BNot => write!(f, "~"),
        }
    }
}

pub enum Builtin {
    Print,
    Println,
    EPrintln,
    EPrint,
    Read,
    Readln,
    Exit,
    ToNumber,
    ToString,
}

impl std::fmt::Display for Builtin {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Builtin::Print => write!(f, "print"),
            Builtin::Println => write!(f, "println"),
            Builtin::EPrintln => write!(f, "eprintln"),
            Builtin::EPrint => write!(f, "eprint"),
            Builtin::Read => write!(f, "read"),
            Builtin::Readln => write!(f, "readln"),
            Builtin::Exit => write!(f, "exit"),
            Builtin::ToNumber => write!(f, "tonumber"),
            Builtin::ToString => write!(f, "tostring"),
        }
    }
}

pub type Stack = VecDeque<Data>;

#[derive(Debug)]
pub enum RuntimeError {
    ProcedureError {
        call: TokenSpan,          // TokenSpan where the procedure was called
        inner: Box<RuntimeError>, // the original error inside the procedure
    },
    StackUnderflow(TokenSpan, String, usize), // when there's too few data on the stack to perform operation
    UnexpectedType(TokenSpan, String, String, String), // when there's an operation tries to operate with an unsupported or an invalid datatype
    InvalidWord(TokenSpan, String), // used when a word doesn't correspond a valid identifier
    ValueError(TokenSpan, String, String, String), // used when a value is invalid or can not be handled
    ProcRedefinition(TokenSpan, String),           // used when a procedure name is already taken
    DefRedefinition(TokenSpan, String),            // used when a definition name is already taken
    EmptyDefinition(TokenSpan, String),            // used when a definition has empty body
}

pub struct Runtime<'a> {
    input: &'a ProgramTree,
    stack: Stack,
    namespace: Namespace<'a>,
    stop: bool,
}

impl<'a> Runtime<'a> {
    pub fn new(input: &'a ProgramTree) -> Self {
        Self {
            input,
            stack: VecDeque::new(),
            namespace: Namespace {
                procs: Vec::new(),
                defs: Vec::new(),
            },
            stop: false,
        }
    }

    fn pre_execution_scan(&mut self) -> Result<(), RuntimeError> {
        for n in self.input {
            match n {
                Node::Proc(n, p, s) => {
                    if let Some(_) = self.namespace.procs.iter().find(|p| p.0 == *n) {
                        return Err(RuntimeError::ProcRedefinition(s.clone(), n.to_string()));
                    }
                    self.namespace.procs.push(Procedure(n.to_string(), p));
                }
                Node::Def(n, p, s) => {
                    if let Some(_) = self.namespace.defs.iter().find(|p| p.0 == *n) {
                        return Err(RuntimeError::DefRedefinition(s.clone(), n.to_string()));
                    }
                    self.run_block(p)?;
                    if let Some(result) = self.pop() {
                        self.namespace.defs.push(Definition(n.to_string(), result));
                    } else {
                        return Err(RuntimeError::EmptyDefinition(s.clone(), n.to_string()));
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn builtin(&mut self, span: TokenSpan, x: Builtin) -> Result<(), RuntimeError> {
        match x {
            Builtin::Println => {
                if let Some(a) = self.pop() {
                    match a {
                        Data::String(s) => {
                            println!("{}", s);
                        }
                        Data::Number(n) => {
                            println!("{}", n);
                        }
                    }
                } else {
                    return Err(RuntimeError::StackUnderflow(span, "println".to_string(), 1));
                }
            }
            Builtin::EPrintln => {
                if let Some(a) = self.pop() {
                    match a {
                        Data::String(s) => {
                            eprintln!("{}", s);
                        }
                        Data::Number(n) => {
                            eprintln!("{}", n);
                        }
                    }
                } else {
                    return Err(RuntimeError::StackUnderflow(
                        span,
                        "eprintln".to_string(),
                        1,
                    ));
                }
            }
            Builtin::EPrint => {
                if let Some(a) = self.pop() {
                    match a {
                        Data::String(s) => {
                            eprint!("{}", s);
                            std::io::stderr().flush().unwrap();
                        }
                        Data::Number(n) => {
                            eprint!("{}", n);
                            std::io::stderr().flush().unwrap();
                        }
                    }
                } else {
                    return Err(RuntimeError::StackUnderflow(span, "eprint".to_string(), 1));
                }
            }
            Builtin::Print => {
                if let Some(a) = self.pop() {
                    match a {
                        Data::String(s) => {
                            print!("{}", s);
                            std::io::stdout().flush().unwrap();
                        }
                        Data::Number(n) => {
                            print!("{}", n);
                            std::io::stdout().flush().unwrap();
                        }
                    }
                } else {
                    return Err(RuntimeError::StackUnderflow(span, "print".to_string(), 1));
                }
            }
            Builtin::Readln => {
                let mut xs = String::new();
                if let Ok(_) = std::io::stdin().read_line(&mut xs) {
                    self.push_string(xs.trim().to_string());
                } else {
                    self.push_number(-1.0);
                }
            }
            Builtin::Read => {
                let mut xs = String::new();
                if let Ok(_) = std::io::stdin().read_to_string(&mut xs) {
                    self.push_string(xs);
                } else {
                    self.push_number(-1.0);
                }
            }
            Builtin::Exit => {
                if let Some(a) = self.pop() {
                    match a {
                        Data::Number(n) => {
                            std::process::exit(n as i32);
                        }
                        _ => {
                            return Err(RuntimeError::UnexpectedType(
                                span,
                                "exit".to_string(),
                                "number".to_string(),
                                format!("{}", a),
                            ));
                        }
                    }
                } else {
                    std::process::exit(0);
                }
            }
            Builtin::ToNumber => {
                if let Some(a) = self.pop() {
                    match a {
                        Data::String(s) => match f64::from_str(&s) {
                            Ok(n) => self.push_number(n),
                            Err(_) => {
                                return Err(RuntimeError::ValueError(
                                    span,
                                    format!("{}", x),
                                    "number".to_string(),
                                    s,
                                ));
                            }
                        },
                        a => {
                            return Err(RuntimeError::UnexpectedType(
                                span,
                                format!("{}", x),
                                "numbers or strings".to_string(),
                                format!("({})", &a),
                            ));
                        }
                    }
                } else {
                    return Err(RuntimeError::StackUnderflow(span, format!("{}", x), 1));
                }
            }
            Builtin::ToString => {
                if let Some(a) = self.pop() {
                    match a {
                        Data::Number(n) => self.push_string(n.to_string()),
                        Data::String(s) => self.push_string(s),
                    }
                } else {
                    return Err(RuntimeError::StackUnderflow(span, format!("{}", x), 1));
                }
            }
        }
        Ok(())
    }

    fn unop(&mut self, span: TokenSpan, x: UnaryOp) -> Result<(), RuntimeError> {
        if let Some(a) = self.pop() {
            match a {
                Data::Number(n) => match x {
                    UnaryOp::Trace => println!("number {}", n),
                    UnaryOp::Dup => {
                        self.push_number(n);
                        self.push_number(n);
                    }
                    UnaryOp::Drop => {},
                    UnaryOp::BNot => self.push_number(!(n as i32) as f64),
                },
                Data::String(s) => match x {
                    UnaryOp::Trace => println!("string \"{}\"", s),
                    UnaryOp::Dup => {
                        self.push_string(s.clone());
                        self.push_string(s);
                    }
                    UnaryOp::Drop => {},
                    UnaryOp::BNot => {
                        return Err(RuntimeError::UnexpectedType(
                            span,
                            format!("{}", x),
                            "number".to_string(),
                            "string".to_string(),
                        ))
                    },
                },
            }
        } else {
            return Err(RuntimeError::StackUnderflow(span, format!("{}", x), 1));
        }
        Ok(())
    }

    fn binop(&mut self, span: TokenSpan, x: BinaryOp) -> Result<(), RuntimeError> {
        if let (Some(a), Some(b)) = (self.pop(), self.pop()) {
            match (a, b) {
                (Data::Number(n1), Data::Number(n2)) => match x {
                    BinaryOp::Add => self.push_number(n1 + n2),
                    BinaryOp::Sub => self.push_number(n1 - n2),
                    BinaryOp::Mul => self.push_number(n1 * n2),
                    BinaryOp::Div => self.push_number(n1 / n2),
                    BinaryOp::Mod => self.push_number(n1 % n2),
                    BinaryOp::Exp => self.push_number(n1.powf(n2)),
                    BinaryOp::Eq => self.push_number((n1 == n2) as i32 as f64),
                    BinaryOp::Ne => self.push_number((n1 != n2) as i32 as f64),
                    BinaryOp::Lt => self.push_number((n1 < n2) as i32 as f64),
                    BinaryOp::Gt => self.push_number((n1 > n2) as i32 as f64),
                    BinaryOp::Le => self.push_number((n1 <= n2) as i32 as f64),
                    BinaryOp::Ge => self.push_number((n1 >= n2) as i32 as f64),
                    BinaryOp::Shl => self.push_number(((n1 as i32) << (n2 as i32)) as f64),
                    BinaryOp::Shr => self.push_number(((n1 as i32) >> (n2 as i32)) as f64),
                    BinaryOp::Bor => self.push_number(((n1 as i32) | (n2 as i32)) as f64),
                    BinaryOp::Band => self.push_number(((n1 as i32) & (n2 as i32)) as f64),
                    BinaryOp::Swap => {
                        self.push_number(n1);
                        self.push_number(n2);
                    }
                    BinaryOp::Over => {
                        self.push_number(n2);
                        self.push_number(n1);
                        self.push_number(n2);
                    }
                },
                (ref i @ Data::String(ref s1), ref j @ Data::String(ref s2)) => match x {
                    BinaryOp::Add => self.push_string(s1.to_owned() + s2),
                    BinaryOp::Eq => self.push_number((s1 == s2) as i32 as f64),
                    BinaryOp::Ne => self.push_number((s1 != s2) as i32 as f64),
                    // other comparison operators are not (should not be) supported for strings
                    // BinaryOp::Lt => self.push_number((s1 < s2) as i32 as f64),
                    // BinaryOp::Gt => self.push_number((s1 > s2) as i32 as f64),
                    // BinaryOp::Le => self.push_number((s1 <= s2) as i32 as f64),
                    // BinaryOp::Ge => self.push_number((s1 >= s2) as i32 as f64),
                    BinaryOp::Swap => {
                        self.push_string(s1.to_string());
                        self.push_string(s2.to_string());
                    }
                    BinaryOp::Over => {
                        self.push_string(s2.clone());
                        self.push_string(s1.clone());
                        self.push_string(s2.to_string());
                    }
                    _ => {
                        return Err(RuntimeError::UnexpectedType(
                            span,
                            format!("{}", x),
                            "numbers".to_string(),
                            format!("({}, {})", i, j),
                        ))
                    }
                },
                (a, b) => {
                    return Err(RuntimeError::UnexpectedType(
                        span,
                        format!("{}", x),
                        "numbers or strings".to_string(),
                        format!("({}, {})", &a, &b),
                    ));
                }
            }
        } else {
            return Err(RuntimeError::StackUnderflow(span, format!("{}", x), 2));
        }
        Ok(())
    }

    fn run_node(&mut self, n: &'a Node) -> Result<(), RuntimeError> {
        match n {
            Node::If(i, e, s) => {
                if let Some(a) = self.pop() {
                    match a {
                        Data::Number(n) => {
                            if n > 0.0 {
                                // negative values or zero = false
                                self.run_block(i)?;
                            } else {
                                if let Some(els) = e {
                                    self.run_block(els)?;
                                }
                            }
                        }
                        Data::String(x) => {
                            if x.len() > 0 {
                                // empty string = false
                                self.run_block(i)?;
                            } else {
                                if let Some(els) = e {
                                    self.run_block(els)?;
                                }
                            }
                        }
                    }
                } else {
                    return Err(RuntimeError::StackUnderflow(s.clone(), "if".to_string(), 1));
                }
            }
            Node::Loop(l, _) => {
                while !self.stop {
                    self.run_block(l)?;

                    if self.stop {
                        self.stop = false;
                        break;
                    }
                }
            }
            Node::Number(n, _) => self.push_number(*n),
            Node::String(v, _) => self.push_string(v.to_string()),
            Node::Operation(op, s) => {
                let s = s.clone(); // TODO: this is a hack, fix it
                match op {
                    OpKind::Add => self.binop(s, BinaryOp::Add)?,
                    OpKind::Sub => self.binop(s, BinaryOp::Sub)?,
                    OpKind::Mul => self.binop(s, BinaryOp::Mul)?,
                    OpKind::Div => self.binop(s, BinaryOp::Div)?,
                    OpKind::Mod => self.binop(s, BinaryOp::Mod)?,
                    OpKind::Exp => self.binop(s, BinaryOp::Exp)?,
                    OpKind::Gt => self.binop(s, BinaryOp::Gt)?,
                    OpKind::Lt => self.binop(s, BinaryOp::Lt)?,
                    OpKind::Eq => self.binop(s, BinaryOp::Eq)?,
                    OpKind::Ge => self.binop(s, BinaryOp::Ge)?,
                    OpKind::Le => self.binop(s, BinaryOp::Le)?,
                    OpKind::Ne => self.binop(s, BinaryOp::Ne)?,
                    OpKind::Shl => self.binop(s, BinaryOp::Shl)?,
                    OpKind::Shr => self.binop(s, BinaryOp::Shr)?,
                    OpKind::Bor => self.binop(s, BinaryOp::Bor)?,
                    OpKind::Band => self.binop(s, BinaryOp::Band)?,
                    OpKind::Swap => self.binop(s, BinaryOp::Swap)?,
                    OpKind::Over => self.binop(s, BinaryOp::Over)?,
                    OpKind::BNot => self.unop(s, UnaryOp::BNot)?,
                    OpKind::Dup => self.unop(s, UnaryOp::Dup)?,
                    OpKind::Drop => self.unop(s, UnaryOp::Drop)?,
                    OpKind::Trace => self.unop(s, UnaryOp::Trace)?,
                    OpKind::Rot => {
                        if let (Some(a), Some(b), Some(c)) = (self.pop(), self.pop(), self.pop()) {
                            self.stack.push_front(b);
                            self.stack.push_front(a);
                            self.stack.push_front(c);
                        } else {
                            return Err(RuntimeError::StackUnderflow(s, "rot".to_string(), 3));
                        }
                        Ok(())
                    }?,
                    OpKind::Stop => {
                        self.stop = true;
                    }
                }
            }
            Node::Word(w, s) => {
                let s = s.clone();
                match w.as_str() {
                    "println" => self.builtin(s, Builtin::Println)?,
                    "print" => self.builtin(s, Builtin::Print)?,
                    "eprint" => self.builtin(s, Builtin::EPrint)?,
                    "eprintln" => self.builtin(s, Builtin::EPrintln)?,
                    "readln" => self.builtin(s, Builtin::Readln)?,
                    "read" => self.builtin(s, Builtin::Read)?,
                    "exit" => self.builtin(s, Builtin::Exit)?,
                    "tostring" => self.builtin(s, Builtin::ToString)?,
                    "tonumber" => self.builtin(s, Builtin::ToNumber)?,
                    _ => {
                        if let Some(p) = self.namespace.procs.iter().find(|p| p.0 == *w) {
                            if let Err(e) = self.run_block(&p.1) {
                                return Err(RuntimeError::ProcedureError {
                                    call: s,
                                    inner: Box::new(e),
                                });
                            }
                        } else if let Some(d) = self.namespace.defs.iter().find(|p| p.0 == *w) {
                            match &d.1 {
                                Data::Number(n) => self.push_number(*n),
                                Data::String(s) => self.push_string(String::from(s)),
                            }
                        } else {
                            return Err(RuntimeError::InvalidWord(s, w.to_string()));
                        }
                    }
                }
            }
            Node::Proc(..) => {}
            Node::Def(..) => {}
        }
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), RuntimeError> {
        self.pre_execution_scan()?;
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

    fn pop(&mut self) -> Option<Data> {
        self.stack.pop_front()
    }
}
