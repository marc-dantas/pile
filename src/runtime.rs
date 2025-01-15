use crate::{
    lexer::TokenSpan,
    parser::{Node, OpKind, ProgramTree},
};
use std::{
    collections::{HashMap, VecDeque},
    io::{Read, Write},
    str::FromStr,
};

#[derive(Debug, Clone)]
pub enum Data {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Nil,
}

impl std::fmt::Display for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Data::String(_) => write!(f, "string"),
            Data::Int(_) => write!(f, "int"),
            Data::Float(_) => write!(f, "float"),
            Data::Bool(_) => write!(f, "bool"),
            Data::Nil => write!(f, "nil"),
        }
    }
}

#[derive(Debug)]
pub struct Procedure<'a>(&'a Vec<Node>);

#[derive(Debug)]
pub struct Definition(Data);

#[derive(Debug)]
pub struct Namespace<'a> {
    pub procs: HashMap<String, Procedure<'a>>,
    pub defs: HashMap<String, Definition>,
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
    ToString,
    ToInt,
    ToFloat,
    TypeOf,
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
            Builtin::ToString => write!(f, "tostring"),
            Builtin::ToInt => write!(f, "toint"),
            Builtin::ToFloat => write!(f, "tofloat"),
            Builtin::TypeOf => write!(f, "typeof"),
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
                procs: HashMap::new(),
                defs: HashMap::new(),
            },
            stop: false,
        }
    }

    fn pre_execution_scan(&mut self) -> Result<(), RuntimeError> {
        for n in self.input {
            match n {
                Node::Proc(n, p, s) => {
                    if self.namespace.procs.contains_key(n) {
                        return Err(RuntimeError::ProcRedefinition(s.clone(), n.to_string()));
                    }
                    self.namespace.procs.insert(n.to_string(), Procedure(p));
                }
                Node::Def(n, p, s) => {
                    if self.namespace.defs.contains_key(n) {
                        return Err(RuntimeError::DefRedefinition(s.clone(), n.to_string()));
                    }
                    self.run_block(p)?;
                    if let Some(result) = self.pop() {
                        self.namespace
                            .defs
                            .insert(n.to_string(), Definition(result));
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
                        Data::Int(n) => {
                            println!("{}", n);
                        }
                        Data::Float(n) => {
                            println!("{}", n);
                        }
                        Data::Bool(n) => {
                            println!("{}", n);
                        }
                        Data::Nil => {
                            println!("nil");
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
                        Data::Int(n) => {
                            eprintln!("{}", n);
                        }
                        Data::Float(n) => {
                            eprintln!("{}", n);
                        }
                        Data::Bool(n) => {
                            eprintln!("{}", n);
                        }
                        Data::Nil => {
                            eprintln!("nil");
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
                        Data::Int(n) => {
                            eprint!("{}", n);
                            std::io::stderr().flush().unwrap();
                        }
                        Data::Float(n) => {
                            eprint!("{}", n);
                            std::io::stderr().flush().unwrap();
                        }
                        Data::Bool(n) => {
                            eprint!("{}", n);
                            std::io::stderr().flush().unwrap();
                        }
                        Data::Nil => {
                            eprint!("nil");
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
                        Data::Int(n) => {
                            print!("{}", n);
                            std::io::stdout().flush().unwrap();
                        }
                        Data::Float(n) => {
                            print!("{}", n);
                            std::io::stdout().flush().unwrap();
                        }
                        Data::Bool(n) => {
                            print!("{}", n);
                            std::io::stdout().flush().unwrap();
                        }
                        Data::Nil => {
                            print!("nil");
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
                    self.push_int(-1);
                }
            }
            Builtin::Read => {
                let mut xs = String::new();
                if let Ok(_) = std::io::stdin().read_to_string(&mut xs) {
                    self.push_string(xs);
                } else {
                    self.push_int(-1);
                }
            }
            Builtin::Exit => {
                if let Some(a) = self.pop() {
                    match a {
                        Data::Int(n) => {
                            std::process::exit(n as i32);
                        }
                        _ => {
                            return Err(RuntimeError::UnexpectedType(
                                span,
                                "exit".to_string(),
                                "int".to_string(),
                                format!("{}", a),
                            ));
                        }
                    }
                } else {
                    std::process::exit(0);
                }
            }
            Builtin::ToInt => {
                if let Some(a) = self.pop() {
                    match a {
                        Data::String(s) => match i64::from_str(&s) {
                            Ok(n) => self.push_int(n),
                            Err(_) => {
                                return Err(RuntimeError::ValueError(
                                    span,
                                    format!("{}", x),
                                    "int".to_string(),
                                    s,
                                ));
                            }
                        },
                        Data::Float(n) => self.push_int(n as i64),
                        Data::Bool(n) => self.push_int(n as i64),
                        Data::Int(n) => self.push_int(n),
                        _ => {
                            return Err(RuntimeError::UnexpectedType(
                                span,
                                "toint".to_string(),
                                "int, float or bool".to_string(),
                                format!("{}", a),
                            ));
                        }
                    }
                } else {
                    return Err(RuntimeError::StackUnderflow(span, format!("{}", x), 1));
                }
            }
            Builtin::ToFloat => {
                if let Some(a) = self.pop() {
                    match a {
                        Data::String(s) => match f64::from_str(&s) {
                            Ok(n) => self.push_float(n),
                            Err(_) => {
                                return Err(RuntimeError::ValueError(
                                    span,
                                    format!("{}", x),
                                    "float".to_string(),
                                    s,
                                ));
                            }
                        },
                        Data::Int(n) => self.push_float(n as f64),
                        Data::Bool(n) => self.push_float(n as i64 as f64),
                        Data::Float(n) => self.push_float(n),
                        _ => {
                            return Err(RuntimeError::UnexpectedType(
                                span,
                                "tofloat".to_string(),
                                "int, float or bool".to_string(),
                                format!("{}", a),
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
                        Data::Int(n) => self.push_string(n.to_string()),
                        Data::Float(n) => self.push_string(n.to_string()),
                        Data::Bool(s) => self.push_string((s as i32).to_string()),
                        Data::String(s) => self.push_string(s),
                        Data::Nil => self.push_string("nil".to_string()),
                    }
                } else {
                    return Err(RuntimeError::StackUnderflow(span, format!("{}", x), 1));
                }
            }
            Builtin::TypeOf => {
                if let Some(a) = self.pop() {
                    self.push_string(format!("{}", a));
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
                Data::Int(n) => match x {
                    UnaryOp::Trace => println!("int {}", n),
                    UnaryOp::Dup => {
                        self.push_int(n);
                        self.push_int(n);
                    }
                    UnaryOp::Drop => {}
                    UnaryOp::BNot => self.push_int(!n),
                },
                Data::Bool(n) => match x {
                    UnaryOp::Trace => println!("bool {}", n),
                    UnaryOp::Dup => {
                        self.push_bool(n);
                        self.push_bool(n);
                    }
                    UnaryOp::Drop => {}
                    UnaryOp::BNot => self.push_bool(!n),
                },
                Data::Float(n) => match x {
                    UnaryOp::Trace => println!("float {}", n),
                    UnaryOp::Dup => {
                        self.push_float(n);
                        self.push_float(n);
                    }
                    UnaryOp::Drop => {}
                    UnaryOp::BNot => self.push_float(!(n as i64) as f64),
                },
                Data::Nil => match x {
                    UnaryOp::Trace => println!("nil"),
                    UnaryOp::Dup => {
                        self.push_nil();
                        self.push_nil();
                    }
                    UnaryOp::Drop => {}
                    _ => {
                        return Err(RuntimeError::UnexpectedType(
                            span,
                            format!("{}", x),
                            "int, float or string".to_string(),
                            "nil".to_string(),
                        ))
                    }
                },
                Data::String(s) => match x {
                    UnaryOp::Trace => println!("string \"{}\"", s),
                    UnaryOp::Dup => {
                        self.push_string(s.clone());
                        self.push_string(s);
                    }
                    UnaryOp::Drop => {}
                    _ => {
                        return Err(RuntimeError::UnexpectedType(
                            span,
                            format!("{}", x),
                            "int or float".to_string(),
                            "string".to_string(),
                        ))
                    }
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
                (Data::Int(n1), Data::Int(n2)) => match x { // TODO: deal with i64 overflows
                    BinaryOp::Add => self.push_int(n1 + n2),
                    BinaryOp::Sub => self.push_int(n1 - n2),
                    BinaryOp::Mul => self.push_int(n1 * n2),
                    BinaryOp::Div => self.push_int(n1 / n2),
                    BinaryOp::Mod => self.push_int(n1 % n2),
                    BinaryOp::Exp => self.push_int(n1.pow(n2 as u32)),
                    BinaryOp::Eq => self.push_bool(n1 == n2),
                    BinaryOp::Ne => self.push_bool(n1 != n2),
                    BinaryOp::Lt => self.push_bool(n1 < n2),
                    BinaryOp::Gt => self.push_bool(n1 > n2),
                    BinaryOp::Le => self.push_bool(n1 <= n2),
                    BinaryOp::Ge => self.push_bool(n1 >= n2),
                    BinaryOp::Shl => self.push_int(n1 << n2),
                    BinaryOp::Shr => self.push_int(n1 >> n2),
                    BinaryOp::Bor => self.push_int(n1 | n2),
                    BinaryOp::Band => self.push_int(n1 & n2),
                    BinaryOp::Swap => {
                        self.push_int(n1);
                        self.push_int(n2);
                    }
                    BinaryOp::Over => {
                        self.push_int(n2);
                        self.push_int(n1);
                        self.push_int(n2);
                    }
                },
                (ref i @ Data::Float(ref n1), ref j @ Data::Float(ref n2)) => match x {
                    BinaryOp::Add => self.push_float(n1 + n2),
                    BinaryOp::Sub => self.push_float(n1 - n2),
                    BinaryOp::Mul => self.push_float(n1 * n2),
                    BinaryOp::Div => self.push_float(n1 / n2),
                    BinaryOp::Mod => self.push_float(n1 % n2),
                    BinaryOp::Exp => self.push_float(n1.powf(*n2)),
                    BinaryOp::Eq => self.push_bool(n1 == n2),
                    BinaryOp::Ne => self.push_bool(n1 != n2),
                    BinaryOp::Lt => self.push_bool(n1 < n2),
                    BinaryOp::Gt => self.push_bool(n1 > n2),
                    BinaryOp::Le => self.push_bool(n1 <= n2),
                    BinaryOp::Ge => self.push_bool(n1 >= n2),
                    BinaryOp::Swap => {
                        self.push_float(*n1);
                        self.push_float(*n2);
                    }
                    BinaryOp::Over => {
                        self.push_float(*n2);
                        self.push_float(*n1);
                        self.push_float(*n2);
                    }
                    _ => {
                        return Err(RuntimeError::UnexpectedType(
                            span,
                            format!("{}", x),
                            "ints".to_string(),
                            format!("({}, {})", i, j),
                        ))
                    }
                },
                (i @ Data::Nil, j @ Data::Nil) => match x {
                    BinaryOp::Swap => {
                        self.stack.push_front(j);
                        self.stack.push_front(i);
                    }
                    BinaryOp::Over => {
                        self.stack.push_front(j.clone());
                        self.stack.push_front(i);
                        self.stack.push_front(j);
                    }
                    _ => {
                        return Err(RuntimeError::UnexpectedType(
                            span,
                            format!("{}", x),
                            "ints, floats, bools or strings".to_string(),
                            format!("({}, {})", i, j),
                        ))
                    }
                },
                (ref i @ Data::String(ref s1), ref j @ Data::String(ref s2)) => match x {
                    BinaryOp::Add => self.push_string(s1.to_owned() + s2),
                    BinaryOp::Eq => self.push_bool(s1 == s2),
                    BinaryOp::Ne => self.push_bool(s1 != s2),
                    BinaryOp::Swap => {
                        self.push_string(s1.to_string());
                        self.push_string(s2.to_string());
                    }
                    BinaryOp::Over => {
                        self.push_string(s2.to_string());
                        self.push_string(s1.to_string());
                        self.push_string(s2.to_string());
                    }
                    _ => {
                        return Err(RuntimeError::UnexpectedType(
                            span,
                            format!("{}", x),
                            "ints or floats".to_string(),
                            format!("({}, {})", i, j),
                        ))
                    }
                },
                (ref i @ Data::Bool(ref b1), ref j @ Data::Bool(ref b2)) => match x {
                    BinaryOp::Eq => self.push_bool(b1 == b2),
                    BinaryOp::Ne => self.push_bool(b1 != b2),
                    BinaryOp::Swap => {
                        self.push_bool(*b1);
                        self.push_bool(*b2);
                    }
                    BinaryOp::Over => {
                        self.push_bool(*b2);
                        self.push_bool(*b1);
                        self.push_bool(*b2);
                    }
                    _ => {
                        return Err(RuntimeError::UnexpectedType(
                            span,
                            format!("{}", x),
                            "ints, floats or strings".to_string(),
                            format!("({}, {})", i, j),
                        ))
                    }
                },
                (a, b) => {
                    return Err(RuntimeError::UnexpectedType(
                        span,
                        format!("{}", x),
                        "ints, floats or strings".to_string(),
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
                        Data::Bool(b) => {
                            if b {
                                self.run_block(i)?;
                            } else {
                                if let Some(els) = e {
                                    self.run_block(els)?;
                                }
                            }
                        }
                        _ => {
                            return Err(RuntimeError::UnexpectedType(
                                s.clone(),
                                "if".to_string(),
                                "bool".to_string(),
                                format!("({})", a),
                            ))
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
            Node::IntLit(n, _) => self.push_int(*n),
            Node::FloatLit(n, _) => self.push_float(*n),
            Node::StringLit(v, _) => self.push_string(v.to_string()),
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
                    OpKind::True => self.push_bool(true),
                    OpKind::False => self.push_bool(false),
                    OpKind::Nil => self.push_nil(),
                    OpKind::IsNil => {
                        if let Some(a) = self.pop() {
                            match a {
                                Data::Nil => self.push_bool(true),
                                _ => self.push_bool(false),
                            }
                        } else {
                            return Err(RuntimeError::StackUnderflow(s, "?".to_string(), 1));
                        }
                    }
                }
            }
            Node::Symbol(w, s) => {
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
                    "toint" => self.builtin(s, Builtin::ToInt)?,
                    "tofloat" => self.builtin(s, Builtin::ToFloat)?,
                    "typeof" => self.builtin(s, Builtin::TypeOf)?,
                    _ => {
                        if let Some(p) = self.namespace.procs.get(w) {
                            if let Err(e) = self.run_block(p.0) {
                                return Err(RuntimeError::ProcedureError {
                                    call: s,
                                    inner: Box::new(e),
                                });
                            }
                        } else if let Some(d) = self.namespace.defs.get(w) {
                            match &d.0 {
                                Data::Int(n) => self.push_int(*n),
                                Data::Float(n) => self.push_float(*n),
                                Data::String(s) => self.push_string(String::from(s)),
                                Data::Bool(b) => self.push_bool(*b),
                                Data::Nil => self.push_nil(),
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

    fn push_int(&mut self, n: i64) {
        self.stack.push_front(Data::Int(n));
    }

    fn push_float(&mut self, n: f64) {
        self.stack.push_front(Data::Float(n));
    }

    fn push_string(&mut self, s: String) {
        self.stack.push_front(Data::String(s));
    }
    
    fn push_bool(&mut self, b: bool) {
        self.stack.push_front(Data::Bool(b));
    }
    
    fn push_nil(&mut self) {
        self.stack.push_front(Data::Nil);
    }

    fn pop(&mut self) -> Option<Data> {
        self.stack.pop_front()
    }
}
