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
pub struct Variable(Data);

#[derive(Debug)]
pub struct Namespace<'a> {
    pub procs: HashMap<&'a str, Procedure<'a>>,
    pub defs: HashMap<&'a str, Definition>,
    pub globals: HashMap<&'a str, Variable>,
    pub locals: HashMap<&'a str, Variable>,
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
        }
    }
}

pub enum UnaryOp {
    BNot,
}

impl std::fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
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

#[allow(dead_code)]
#[derive(Debug, Clone)]
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
    UnboundVariable(TokenSpan, String),            // used when a definition has empty body
}

pub struct Runtime<'a> {
    input: &'a ProgramTree,
    stack: Stack,
    namespace: Namespace<'a>,
    loop_break: bool,
    loop_continue: bool,
    proc_return: bool,
}

// TODO: Optimize stack operations using the peek function.
// This way it is possible to avoid popping and pushing back the same item
// For example (over operation)
// over ( a b -- a b a )
// For now, I pop 2 items on top of the stack and push them back in the desired order after the operation
// But i can just peek(1) (second last element) and then push it to the top. Way better.
impl<'a> Runtime<'a> {
    pub fn new(input: &'a ProgramTree) -> Self {
        Self {
            input,
            stack: VecDeque::new(),
            namespace: Namespace {
                procs: HashMap::new(),
                defs: HashMap::new(),
                globals: HashMap::new(),
                locals: HashMap::new(),
            },
            loop_break: false,
            loop_continue: false,
            proc_return: false,
        }
    }

    fn pre_execution_scan(&mut self) -> Result<(), RuntimeError> {
        for n in self.input {
            match n {
                Node::Proc(n, p, s) => {
                    if self.namespace.procs.contains_key(n.as_str()) {
                        return Err(RuntimeError::ProcRedefinition(s.clone(), n.to_string()));
                    }
                    self.namespace.procs.insert(n, Procedure(p));
                }
                Node::Def(n, p, s) => {
                    if self.namespace.defs.contains_key(n.as_str()) {
                        return Err(RuntimeError::DefRedefinition(s.clone(), n.to_string()));
                    }
                    self.run_block(p)?;
                    if let Some(result) = self.pop() {
                        self.namespace
                            .defs
                            .insert(n, Definition(result));
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
                            Err(_) => self.push_nil(),
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
                            Err(_) => self.push_nil(),
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
                    UnaryOp::BNot => self.push_int(!n),
                },
                Data::Bool(n) => match x {
                    UnaryOp::BNot => self.push_bool(!n),
                },
                Data::Float(n) => match x {
                    UnaryOp::BNot => self.push_float(!(n as i64) as f64),
                },
                Data::Nil => match x {
                    _ => {
                        return Err(RuntimeError::UnexpectedType(
                            span,
                            format!("{}", x),
                            "int, float or string".to_string(),
                            "nil".to_string(),
                        ))
                    }
                },
                Data::String(_s) => match x {
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
                (Data::Int(n1), Data::Int(n2)) => match x {
                    // TODO: deal with i64 overflows
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
                'outer: loop {
                    for n in l {
                        self.run_node(n)?;
                        if self.loop_break {
                            break 'outer;
                        }
                        if self.loop_continue {
                            // The reason why I don't use 'outer label is that it would re-execute
                            // the entire block of the loop if I used the label.
                            continue;
                        }
                    }
                }
                self.loop_break = false;
                self.loop_continue = false;
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
                    OpKind::BNot => self.unop(s, UnaryOp::BNot)?,
                    OpKind::Swap => {
                        if let (Some(x), Some(y)) = (self.pop(), self.pop()) {
                            self.stack.push_front(x);
                            self.stack.push_front(y);
                        } else {
                            return Err(RuntimeError::StackUnderflow(s, "swap".to_string(), 2));
                        }
                        Ok(())
                    }?,
                    OpKind::Over => {
                        if let Some(x) = self.peek(1) {
                            self.stack.push_front(x.clone()); // TODO: Make Data a stack structure
                        } else {
                            return Err(RuntimeError::StackUnderflow(s, "over".to_string(), 2));
                        }
                        Ok(())
                    }?,
                    OpKind::Drop => {
                        if let None = self.pop() {
                            return Err(RuntimeError::StackUnderflow(s, "drop".to_string(), 1));
                        }
                        Ok(())
                    }?,
                    OpKind::Dup => {
                        if let Some(a) = self.peek(0) {
                            self.stack.push_front(a.clone()); // TODO: Make Data a stack structure
                        } else {
                            return Err(RuntimeError::StackUnderflow(s, "dup".to_string(), 1));
                        }
                        Ok(())
                    }?,
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
                    OpKind::Trace => {
                        if let Some(a) = self.peek(0) {
                            match a {
                                Data::Int(n) => println!("int {}", n),
                                Data::Float(n) => println!("float {}", n),
                                Data::String(s) => println!("string \"{}\"", s),
                                Data::Bool(b) => println!("bool {}", b),
                                Data::Nil => println!("nil"),
                            }
                        } else {
                            return Err(RuntimeError::StackUnderflow(s, "trace".to_string(), 1));
                        }
                    }
                    OpKind::Break => self.loop_break = true,
                    OpKind::Continue => self.loop_continue = true,
                    OpKind::Return => self.proc_return = true,
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
                        if let Some(p) = self.namespace.procs.get(w.as_str()) {
                            for n in p.0 {
                                if let Err(e) = self.run_node(n) {
                                    return Err(RuntimeError::ProcedureError {
                                        call: s,
                                        inner: Box::new(e),
                                    });
                                }
                                if self.proc_return { break; }
                            }
                            self.proc_return = false;
                        } else if let Some(d) = self.namespace.defs.get(w.as_str()) {
                            self.stack.push_front(d.0.clone()); // TODO: Make Data a stack structure
                        } else if let Some(v) = self.namespace.globals.get(w.as_str()) {
                            self.stack.push_front(v.0.clone()); // TODO: Make Data a stack structure
                        } else if let Some(v) = self.namespace.locals.get(w.as_str()) {
                            self.stack.push_front(v.0.clone()); // TODO: Make Data a stack structure
                        } else {
                            return Err(RuntimeError::InvalidWord(s, w.to_string()));
                        }
                    }
                }
            }
            Node::Let(name, span) => {
                if let Some(a) = self.pop() {
                    if let Some(_) = self.namespace.locals.get(name.as_str()) {
                        self.namespace.locals.insert(name, Variable(a));
                    } else {
                        self.namespace.globals.insert(name, Variable(a));
                    }
                } else {
                    return Err(RuntimeError::UnboundVariable(
                        span.clone(),
                        name.to_string(),
                    ));
                }
            }
            Node::AsLet(vars, block, _) => {
                let mut defined_vars = Vec::new();
                for x in vars.into_iter().rev() {
                    defined_vars.push(&x.value);
                    if let Some(a) = self.pop() {
                        self.namespace.locals.insert(&x.value, Variable(a));
                    } else {
                        return Err(RuntimeError::UnboundVariable(
                            x.span.clone(),
                            x.value.to_string(),
                        ));
                    }
                }
                self.run_block(block)?;
                for var in defined_vars.iter() {
                    self.namespace.locals.remove(var.as_str());
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

    // Peek operation is meant to get an item at the nth position starting from the top
    // without afecting the stack
    fn peek(&self, nth: usize) -> Option<&Data> {
        self.stack.get(nth)
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
