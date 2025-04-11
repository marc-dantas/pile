use crate::{
    error::{self, fatal}, lexer::{FileSpan, Span}, parser::{Node, OpKind, ProgramTree}
};
use std::{
    collections::{HashMap, VecDeque}, io::{Read, Write}, str::FromStr
};

#[derive(Debug, Clone, Copy)]
pub enum Data {
    Array(u32),
    String(usize, usize),
    Int(i64),
    Float(f64),
    Bool(bool),
    Nil,
}

impl Data {
    fn format(&self) -> &str {
        match *self {
            Data::String(..) => "string",
            Data::Int(_)     => "int",
            Data::Float(_)   => "float",
            Data::Bool(_)    => "bool",
            Data::Nil        => "nil",
            Data::Array(_)   => "array",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Procedure<'a>(&'a Vec<Node>, FileSpan);

#[derive(Debug, Clone)]
pub struct Definition(Data);

#[derive(Debug, Clone)]
pub struct Variable(Data);

#[derive(Debug, Clone)]
pub struct Namespace<'a> {
    pub procs: HashMap<&'a str, Procedure<'a>>,
    pub defs: HashMap<&'a str, Definition>,
    pub globals: HashMap<&'a str, Variable>,
    pub locals: Vec<HashMap<&'a str, Variable>>,
}

impl<'a> Namespace<'a> {
    pub fn extend(&mut self, other: Namespace<'a>) {
        self.procs.extend(other.procs);
        self.defs.extend(other.defs);
        self.globals.extend(other.globals);
        // Locals don't need to be extended since they are locals! duuuh
    }
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
    Input,
    InputLn,
    Exit,
    ToString,
    ToInt,
    ToFloat,
    TypeOf,
    Chr,
    Len,
}

impl std::fmt::Display for Builtin {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Builtin::Print => write!(f, "print"),
            Builtin::Println => write!(f, "println"),
            Builtin::EPrintln => write!(f, "eprintln"),
            Builtin::EPrint => write!(f, "eprint"),
            Builtin::Input => write!(f, "input"),
            Builtin::InputLn => write!(f, "inputln"),
            Builtin::Exit => write!(f, "exit"),
            Builtin::ToString => write!(f, "tostring"),
            Builtin::ToInt => write!(f, "toint"),
            Builtin::ToFloat => write!(f, "tofloat"),
            Builtin::TypeOf => write!(f, "typeof"),
            Builtin::Chr => write!(f, "chr"),
            Builtin::Len => write!(f, "len")
        }
    }
}

pub type Stack<'a> = VecDeque<Data>;

// #[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum RuntimeError {
    ProcedureError {
        call: FileSpan,           // TokenSpan where the procedure was called
        inner: Box<RuntimeError>, // the original error inside the procedure
    },
    RecursionDepthOverflow(FileSpan),                 // when a procedure recurses too many times
    ImportError(FileSpan, String),                    // when it's not possible to import
    StackUnderflow(FileSpan, String, usize),          // when there's too few data on the stack to perform operation
    UnexpectedType(FileSpan, String, String, String), // when there's an operation tries to operate with an invalid datatype
    InvalidWord(FileSpan, String),                    // used when a word isn't defined
    ValueError(FileSpan, String, String, String),     // when a value is invalid or can not be handled
    ProcRedefinition(FileSpan, String),               // when a procedure name is already taken
    DefRedefinition(FileSpan, String),                // when a definition name is already taken
    EmptyDefinition(FileSpan, String),                // when a definition has empty body
    UnboundVariable(FileSpan, String),                // when a variable has no value
    ReadMemoryOutOfBounds(FileSpan, usize),           // when tries to read outside of memory bounds
    WriteMemoryOutOfBounds(FileSpan, String, usize),  // when tries to write outside of memory bounds
    ArrayOutOfBounds(FileSpan, i64, usize),         // when tries to index array at invalid index
    StringOutOfBounds(FileSpan, i64, usize)         // when tries to index string at invalid index
}

pub fn parse_file(filename: &str, source: String) -> Result<ProgramTree, crate::parser::ParseError> {
    let f = crate::lexer::InputFile {
        name: filename,
        content: source.chars().peekable(),
    };
    let l = crate::lexer::Lexer::new(f, Span { line: 1, col: 1 });
    let mut p = crate::parser::Parser::new(l);
    p.parse()
}

pub fn import<'a>(program: &'a Vec<Node>, path: &'a str) -> Option<Namespace<'a>> {
    let mut runtime = Runtime::new(&program, path);
    match runtime.run() {
        Ok(()) => {
            return Some(runtime.namespace);
        }
        Err(e) => error::runtime_error(e),
    }
    None
}

pub const STR_CAPACITY: usize = 100*1024; // 100kb of strings should be enough
pub const MAX_RECURSION_DEPTH: u32 = 256; // TODO: Make thread stack bigger to allow deeper recursion

pub struct Runtime<'a> {
    input: &'a ProgramTree,
    root_filename: &'a str,
    filename: &'a str,
    string_buffer: [u8; STR_CAPACITY],
    string_ptr: usize,
    arrays: HashMap<u32, Vec<Data>>,
    current_array: u32,
    stack: Stack<'a>,
    namespace: Namespace<'a>,
    loop_break: bool,
    loop_continue: bool,
    proc_return: bool,
    recursion_depth: u32,
}

impl<'a> Runtime<'a> {
    pub fn new(input: &'a ProgramTree, filename: &'a str) -> Self {
        Self {
            input, filename,
            root_filename: filename,
            string_buffer: [0; STR_CAPACITY],
            string_ptr: 0,
            arrays: HashMap::new(),
            current_array: 0,
            stack: VecDeque::new(),
            namespace: Namespace {
                procs: HashMap::new(),
                defs: HashMap::new(),
                globals: HashMap::new(),
                locals: Vec::new(),
            },
            loop_break: false,
            loop_continue: false,
            proc_return: false,
            recursion_depth: 0,
        }
    }

    fn prepopulate_namespace(&mut self) -> Result<(), RuntimeError> {
        for n in self.input {
            match n {
                Node::Import(path, span) => {
                    match std::fs::File::open(path) {
                        Ok(mut f) => {
                            let mut xs = Vec::new();
                            f.read_to_end(&mut xs).unwrap();
                            match String::from_utf8(xs) {
                                _ if path == self.filename => return Err(RuntimeError::ImportError(
                                    span.to_filespan(self.filename.to_string()),
                                    path.clone()
                                )),
                                Ok(source) => {
                                    let parsed = parse_file(path, source);
                                    match parsed {
                                        Ok(ast) => {
                                            let x = import(Box::leak(Box::new(ast)), path).unwrap();
                                            self.namespace.extend(x);
                                        }
                                        Err(e) => error::parse_error(e),
                                    }
                                },
                                Err(_) => return Err(RuntimeError::ImportError(
                                    span.to_filespan(self.filename.to_string()),
                                    path.clone()
                                )),
                            }
                        }
                        Err(_) => return Err(RuntimeError::ImportError(
                            span.to_filespan(self.filename.to_string()),
                            path.clone()
                        )),
                    }
                }
                Node::Proc(n, p, s) => {
                    if self.namespace.procs.contains_key(n.as_str()) {
                        return Err(RuntimeError::ProcRedefinition(s.to_filespan(self.filename.to_string()), n.to_string()));
                    }
                    self.namespace.procs.insert(n, Procedure(p, s.to_filespan(self.filename.to_string())));
                }
                Node::Def(n, p, s) => {
                    if self.namespace.defs.contains_key(n.as_str()) {
                        return Err(RuntimeError::DefRedefinition(s.to_filespan(self.filename.to_string()), n.to_string()));
                    }
                    self.run_block(p)?;
                    if let Some(result) = self.stack.pop_front() {
                        self.namespace
                            .defs
                            .insert(n, Definition(result));
                    } else {
                        return Err(RuntimeError::EmptyDefinition(s.to_filespan(self.filename.to_string()), n.to_string()));
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn builtin(&mut self, span: Span, x: Builtin) -> Result<(), RuntimeError> {
        match x {
            Builtin::Println => {
                if let Some(a) = self.pop() {
                    match a {
                        Data::String(ptr, size) => {
                            let s = self.read_string(ptr, size);
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
                        _ => {
                            return Err(RuntimeError::UnexpectedType(
                                span.to_filespan(self.filename.to_string()),
                                "println".to_string(),
                                "string, int, float, bool or nil".to_string(),
                                a.format().to_string(),
                            ));
                        }
                    }
                } else {
                    return Err(RuntimeError::StackUnderflow(span.to_filespan(self.filename.to_string()), "println".to_string(), 1));
                }
            }
            Builtin::EPrintln => {
                if let Some(a) = self.pop() {
                    match a {
                        Data::String(ptr, size) => {
                            let s = self.read_string(ptr, size);
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
                        _ => {
                            return Err(RuntimeError::UnexpectedType(
                                span.to_filespan(self.filename.to_string()),
                                "eprintln".to_string(),
                                "string, int, float, bool or nil".to_string(),
                                a.format().to_string(),
                            ));
                        }
                    }
                } else {
                    return Err(RuntimeError::StackUnderflow(
                        span.to_filespan(self.filename.to_string()),
                        "eprintln".to_string(),
                        1,
                    ));
                }
            }
            Builtin::EPrint => {
                if let Some(a) = self.pop() {
                    match a {
                        Data::String(ptr, size) => {
                            let s = self.read_string(ptr, size);
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
                        _ => {
                            return Err(RuntimeError::UnexpectedType(
                                span.to_filespan(self.filename.to_string()),
                                "eprint".to_string(),
                                "string, int, float, bool or nil".to_string(),
                                a.format().to_string(),
                            ));
                        }
                    }
                } else {
                    return Err(RuntimeError::StackUnderflow(span.to_filespan(self.filename.to_string()), "eprint".to_string(), 1));
                }
            }
            Builtin::Print => {
                if let Some(a) = self.pop() {
                    match a {
                        Data::String(ptr, size) => {
                            let s = self.read_string(ptr, size);
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
                        _ => {
                            return Err(RuntimeError::UnexpectedType(
                                span.to_filespan(self.filename.to_string()),
                                "print".to_string(),
                                "string, int, float, bool or nil".to_string(),
                                a.format().to_string(),
                            ));
                        }
                    }
                } else {
                    return Err(RuntimeError::StackUnderflow(span.to_filespan(self.filename.to_string()), "print".to_string(), 1));
                }
            }
            Builtin::InputLn => {
                let mut xs = String::new();
                if let Ok(_) = std::io::stdin().read_line(&mut xs) {
                    self.push_string(xs.trim().to_string().as_bytes());
                } else {
                    self.push_int(-1);
                }
            }
            Builtin::Input => {
                let mut xs = String::new();
                if let Ok(_) = std::io::stdin().read_to_string(&mut xs) {
                    self.push_string(xs.as_bytes());
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
                                span.to_filespan(self.filename.to_string()),
                                "exit".to_string(),
                                "int".to_string(),
                                a.format().to_string(),
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
                        Data::String(ptr, size) => {
                            let s = self.read_string(ptr, size);
                            match i64::from_str(s) {
                                Ok(n) => self.push_int(n),
                                Err(_) => self.push_nil(),
                            }
                        },
                        Data::Float(n) => self.push_int(n as i64),
                        Data::Bool(n) => self.push_int(n as i64),
                        Data::Int(n) => self.push_int(n),
                        _ => {
                            return Err(RuntimeError::UnexpectedType(
                                span.to_filespan(self.filename.to_string()),
                                "toint".to_string(),
                                "int, float or bool".to_string(),
                                a.format().to_string(),
                            ));
                        }
                    }
                } else {
                    return Err(RuntimeError::StackUnderflow(span.to_filespan(self.filename.to_string()), format!("{}", x), 1));
                }
            }
            Builtin::ToFloat => {
                if let Some(a) = self.pop() {
                    match a {
                        Data::String(ptr, size) => {
                            let s = self.read_string(ptr, size);
                            match f64::from_str(&s) {
                                Ok(n) => self.push_float(n),
                                Err(_) => self.push_nil(),
                            }
                        },
                        Data::Int(n) => self.push_float(n as f64),
                        Data::Bool(n) => self.push_float(n as i64 as f64),
                        Data::Float(n) => self.push_float(n),
                        _ => {
                            return Err(RuntimeError::UnexpectedType(
                                span.to_filespan(self.filename.to_string()),
                                "tofloat".to_string(),
                                "int, float or bool".to_string(),
                                a.format().to_string(),
                            ));
                        }
                    }
                } else {
                    return Err(RuntimeError::StackUnderflow(span.to_filespan(self.filename.to_string()), format!("{}", x), 1));
                }
            }
            Builtin::ToString => {
                if let Some(a) = self.pop() {
                    match a {
                        Data::String(..) => self.stack.push_front(a),
                        Data::Int(i) => self.push_string(i.to_string().as_bytes()),
                        Data::Float(f) => self.push_string(f.to_string().as_bytes()),
                        Data::Bool(b) => self.push_string(b.to_string().as_bytes()),
                        Data::Nil => self.push_string("nil".to_string().as_bytes()),
                        _ => {
                            return Err(RuntimeError::UnexpectedType(
                                span.to_filespan(self.filename.to_string()),
                                "tostring".to_string(),
                                "string, int, float, bool or nil".to_string(),
                                a.format().to_string(),
                            ));
                        }
                    }
                } else {
                    return Err(RuntimeError::StackUnderflow(span.to_filespan(self.filename.to_string()), format!("{}", x), 1));
                }
            }
            Builtin::TypeOf => {
                if let Some(a) = self.stack.pop_front() {
                    self.push_string(a.format().as_bytes());
                } else {
                    return Err(RuntimeError::StackUnderflow(span.to_filespan(self.filename.to_string()), format!("{}", x), 1));
                }
            }
            Builtin::Chr => {
                if let Some(a) = self.pop() {
                    match a {
                        Data::Int(c) => {
                            let mut c = c;
                            if c > 255 { c = 0;   }
                            if c < 0   { c = 255; }
                            self.push_string(&[c as u8]);
                        }
                        _ => return Err(RuntimeError::UnexpectedType(
                            span.to_filespan(self.filename.to_string()),
                            "chr".to_string(),
                            "int".to_string(),
                            format!("{}", a.format()),
                        )),
                    }
                } else {
                    return Err(RuntimeError::StackUnderflow(span.to_filespan(self.filename.to_string()), "chr".to_string(), 1));
                }
            }
            Builtin::Len => {
                if let Some(a) = self.pop() {
                    match a {
                        Data::String(addr, size) => {
                            let str = self.read_string(addr, size);
                            self.push_int(str.len() as i64);
                        }
                        Data::Array(id) => {
                            let arr = self.arrays.get(&id).unwrap();
                            self.push_int(arr.len() as i64);
                        }
                        _ => return Err(RuntimeError::UnexpectedType(
                            span.to_filespan(self.filename.to_string()),
                            "len".to_string(),
                            "string or array".to_string(),
                            format!("{}", a.format()),
                        )),
                    }
                } else {
                    return Err(RuntimeError::StackUnderflow(span.to_filespan(self.filename.to_string()), "len".to_string(), 1));
                }
            }
        }
        Ok(())
    }

    fn unop(&mut self, span: Span, x: UnaryOp) -> Result<(), RuntimeError> {
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
                _ => {
                    return Err(RuntimeError::UnexpectedType(
                        span.to_filespan(self.filename.to_string()),
                        format!("{}", x),
                        "int, float or bool".to_string(),
                        a.format().to_string(),
                    ));
                }
            }
        } else {
            return Err(RuntimeError::StackUnderflow(span.to_filespan(self.filename.to_string()), format!("{}", x), 1));
        }
        Ok(())
    }

    fn binop(&mut self, span: Span, x: BinaryOp) -> Result<(), RuntimeError> {
        if let (Some(a), Some(b)) = (self.pop(), self.pop()) {
            match (b, a) { // Inverted stack order to match the writing order
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
                            span.to_filespan(self.filename.to_string()),
                            format!("{}", x),
                            "ints, bools, strings or arrays".to_string(),
                            format!("({}, {})", i.format(), j.format()),
                        ))
                    }
                },
                (i @ Data::Nil, j @ Data::Nil) => match x {
                    _ => {
                        return Err(RuntimeError::UnexpectedType(
                            span.to_filespan(self.filename.to_string()),
                            format!("{}", x),
                            "ints, floats, bools, strings or arrays".to_string(),
                            format!("({}, {})", i.format(), j.format()),
                        ))
                    }
                },
                (ref i @ Data::String(ptr1, size1), ref j @ Data::String(ptr2, size2)) => {
                    let s1 = self.read_string(ptr1, size1);
                    let s2 = self.read_string(ptr2, size2);
                    match x {
                        BinaryOp::Add => self.push_string(((*s1).to_owned() + s2).as_bytes()),
                        BinaryOp::Eq => self.push_bool(s1 == s2),
                        BinaryOp::Ne => self.push_bool(s1 != s2),
                        _ => {
                            return Err(RuntimeError::UnexpectedType(
                                span.to_filespan(self.filename.to_string()),
                                format!("{}", x),
                                "ints, floats, bools or arrays".to_string(),
                                format!("({}, {})", i.format(), j.format()),
                            ))
                        }
                    }
                }
                (ref i @ Data::Bool(ref b1), ref j @ Data::Bool(ref b2)) => match x {
                    BinaryOp::Eq => self.push_bool(b1 == b2),
                    BinaryOp::Ne => self.push_bool(b1 != b2),
                    BinaryOp::Bor => self.push_bool(*b1 || *b2),
                    BinaryOp::Band => self.push_bool(*b1 && *b2),
                    _ => {
                        return Err(RuntimeError::UnexpectedType(
                            span.to_filespan(self.filename.to_string()),
                            format!("{}", x),
                            "ints, floats, strings or arrays".to_string(),
                            format!("({}, {})", i.format(), j.format()),
                        ))
                    }
                },
                (ref i @ Data::Array(id1), ref j @ Data::Array(id2)) => match x {
                    BinaryOp::Add => {
                        let mut new_arr = Vec::new();
                        let arr1 = self.load_array(id1);
                        new_arr.extend(arr1);
                        let arr2 = self.load_array(id2);
                        new_arr.extend(arr2);
                        self.push_array(new_arr);
                    },
                    _ => {
                        return Err(RuntimeError::UnexpectedType(
                            span.to_filespan(self.filename.to_string()),
                            format!("{}", x),
                            "ints, floats, bools or strings".to_string(),
                            format!("({}, {})", i.format(), j.format()),
                        ))
                    }
                },
                (a, b) => {
                    return Err(RuntimeError::UnexpectedType(
                        span.to_filespan(self.filename.to_string()),
                        format!("{}", x),
                        "any other type".to_string(),
                        format!("({}, {})", a.format(), b.format()),
                    ));
                }
            }
        } else {
            return Err(RuntimeError::StackUnderflow(span.to_filespan(self.filename.to_string()), format!("{}", x), 2));
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
                                s.to_filespan(self.filename.to_string()),
                                "if".to_string(),
                                "bool".to_string(),
                                format!("({})", a.format()),
                            ))
                        }
                    }
                } else {
                    return Err(RuntimeError::StackUnderflow(s.to_filespan(self.filename.to_string()), "if".to_string(), 1));
                }
            }
            Node::Loop(l, _) => {
                self.loop_break = false;
                self.loop_continue = false;
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
            Node::StringLit(v, _) => self.push_string(v.as_bytes()),
            Node::Operation(op, s) => {
                let s = *s;
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
                            return Err(RuntimeError::StackUnderflow(s.to_filespan(self.filename.to_string()), "swap".to_string(), 2));
                        }
                        Ok(())
                    }?,
                    OpKind::Over => {
                        if let Some(x) = self.peek(1) {
                            self.stack.push_front(*x);
                        } else {
                            return Err(RuntimeError::StackUnderflow(s.to_filespan(self.filename.to_string()), "over".to_string(), 2));
                        }
                        Ok(())
                    }?,
                    OpKind::Drop => {
                        if let None = self.pop() {
                            return Err(RuntimeError::StackUnderflow(s.to_filespan(self.filename.to_string()), "drop".to_string(), 1));
                        }
                        Ok(())
                    }?,
                    OpKind::Dup => {
                        if let Some(a) = self.peek(0) {
                            self.stack.push_front(*a);
                        } else {
                            return Err(RuntimeError::StackUnderflow(s.to_filespan(self.filename.to_string()), "dup".to_string(), 1));
                        }
                        Ok(())
                    }?,
                    OpKind::Rot => {
                        if let (Some(a), Some(b), Some(c)) = (self.pop(), self.pop(), self.pop()) {
                            self.stack.push_front(b);
                            self.stack.push_front(a);
                            self.stack.push_front(c);
                        } else {
                            return Err(RuntimeError::StackUnderflow(s.to_filespan(self.filename.to_string()), "rot".to_string(), 3));
                        }
                        Ok(())
                    }?,
                    OpKind::Trace => {
                        if let Some(a) = self.peek(0) {
                            match a {
                                Data::Int(n) => println!("int {}", n),
                                Data::Float(n) => println!("float {}", n),
                                Data::String(ptr, size) => {
                                    let s = self.read_string(*ptr, *size);
                                    println!("string \"{}\"", s)
                                },
                                Data::Bool(b) => println!("bool {}", b),
                                Data::Nil => println!("nil"),
                                Data::Array(id) => println!("array at 0X{:X}", id),
                            }
                        } else {
                            return Err(RuntimeError::StackUnderflow(s.to_filespan(self.filename.to_string()), "trace".to_string(), 1));
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
                            return Err(RuntimeError::StackUnderflow(s.to_filespan(self.filename.to_string()), "?".to_string(), 1));
                        }
                    }
                    OpKind::SeqIndex => {
                        if let (Some(a), Some(b)) = (self.pop(), self.pop()) {
                            match (a, b) {
                                (Data::Int(index), Data::Array(id)) => {
                                    let mut i = index.abs() as usize;
                                    let arr = self.arrays.get(&id).unwrap();
                                    if index < 0 { i = arr.len() - i; }
                                    if let Some(d) = arr.iter().nth(i) {
                                        self.stack.push_front(*d);
                                    } else {
                                        return Err(RuntimeError::ArrayOutOfBounds(
                                            s.to_filespan(self.filename.to_string()),
                                            index,
                                            arr.len(),
                                        ));
                                    }
                                }
                                (Data::Int(index), Data::String(ptr, size)) => {
                                    let mut i = index.abs() as usize;
                                    let str = self.read_string(ptr, size).as_bytes();
                                    if index < 0 { i = str.len() - i; }
                                    if i >= str.len() {
                                        return Err(RuntimeError::StringOutOfBounds(
                                            s.to_filespan(self.filename.to_string()),
                                            index,
                                            str.len(),
                                        ));
                                    }
                                    // self.push_string(&[str[i]]);
                                    self.stack.push_front(Data::String(ptr+i, 1));
                                }
                                (a, b) => {
                                    return Err(RuntimeError::UnexpectedType(
                                        s.to_filespan(self.filename.to_string()),
                                        "@".to_string(),
                                        "(array, int) or (string, int)".to_string(),
                                        format!("({}, {})", a.format(), b.format()),
                                    ));
                                }
                            }
                        } else {
                            return Err(RuntimeError::StackUnderflow(s.to_filespan(self.filename.to_string()), "@".to_string(), 2));
                        }
                    }
                    OpKind::SeqAssignAtIndex => {
                        if let (Some(a), Some(b), Some(c)) = (self.pop(), self.pop(), self.pop()) {
                            match (a, b, c) {
                                (Data::Int(index), x, Data::Array(id)) => {
                                    let mut i = index.abs() as usize;
                                    let arr = self.arrays.get_mut(&id).unwrap();
                                    if index < 0 { i = arr.len() - i; }
                                    if let Some(elem) = arr.get_mut(i) {
                                        *elem = x;
                                    } else {
                                        return Err(RuntimeError::ArrayOutOfBounds(
                                            s.to_filespan(self.filename.to_string()),
                                            index,
                                            arr.len(),
                                        ));
                                    }
                                }
                                (Data::Int(index), Data::Int(chr), Data::String(ptr, size)) => {
                                    let mut i = index.abs() as usize;
                                    let str = self.read_string(ptr, size).as_bytes();
                                    if index < 0 { i = str.len() - i; }
                                    if i >= str.len() {
                                        return Err(RuntimeError::StringOutOfBounds(
                                            s.to_filespan(self.filename.to_string()),
                                            index,
                                            str.len(),
                                        ));
                                    }
                                    self.string_buffer[ptr+i] = chr as u8;
                                }
                                (a, b, c) => {
                                    return Err(RuntimeError::UnexpectedType(
                                        s.to_filespan(self.filename.to_string()),
                                        "!".to_string(),
                                        "(array, any, int), (string, int, int) or (string, string, int)".to_string(),
                                        format!("({}, {}, {})", a.format(), b.format(), c.format()),
                                    ));
                                }
                            }
                        } else {
                            return Err(RuntimeError::StackUnderflow(s.to_filespan(self.filename.to_string()), "!".to_string(), 2));
                        }
                    }
                }
            }
            Node::Symbol(w, s) => {
                let s = *s;
                match w.as_str() {
                    "println" => self.builtin(s, Builtin::Println)?,
                    "print" => self.builtin(s, Builtin::Print)?,
                    "eprint" => self.builtin(s, Builtin::EPrint)?,
                    "eprintln" => self.builtin(s, Builtin::EPrintln)?,
                    "inputln" => self.builtin(s, Builtin::InputLn)?,
                    "input" => self.builtin(s, Builtin::Input)?,
                    "exit" => self.builtin(s, Builtin::Exit)?,
                    "tostring" => self.builtin(s, Builtin::ToString)?,
                    "toint" => self.builtin(s, Builtin::ToInt)?,
                    "tofloat" => self.builtin(s, Builtin::ToFloat)?,
                    "typeof" => self.builtin(s, Builtin::TypeOf)?,
                    "chr" => self.builtin(s, Builtin::Chr)?,
                    "len" => self.builtin(s, Builtin::Len)?,
                    _ => {
                        if let Some(p) = self.namespace.procs.get(w.as_str()) {
                            self.proc_return = false;
                            let prev_filename = self.filename;
                            self.filename = Box::leak(Box::new(p.1.filename.clone()));
                            self.recursion_depth += 1;
                            if self.recursion_depth >= MAX_RECURSION_DEPTH {
                                return Err(RuntimeError::RecursionDepthOverflow(p.1.clone()));
                            }
                            for n in p.0 {
                                if let Err(e) = self.run_node(n) {
                                    return Err(RuntimeError::ProcedureError {
                                        call: s.to_filespan(prev_filename.to_string()),
                                        inner: Box::new(e),
                                    });
                                }
                                if self.proc_return { break; }
                            }
                            self.recursion_depth -= 1;
                            self.filename = self.root_filename;
                            self.proc_return = false;
                        } else if let Some(d) = self.namespace.defs.get(w.as_str()) {
                            self.stack.push_front(d.0);
                        } else if let Some(scope) = self.namespace.locals.last_mut() {
                            if let Some(v) = scope.get(w.as_str()) {
                                self.stack.push_front(v.0);
                            } else {
                                return Err(RuntimeError::InvalidWord(s.to_filespan(self.filename.to_string()), w.to_string()));
                            }
                        } else if let Some(v) = self.namespace.globals.get(w.as_str()) {
                            self.stack.push_front(v.0);
                        } else {
                            return Err(RuntimeError::InvalidWord(s.to_filespan(self.filename.to_string()), w.to_string()));
                        }
                    }
                }
            }
            Node::Array(block, _) => {
                let before_len = self.stack.len();
                self.run_block(block)?;
                let after_len = self.stack.len();
                let mut array_len = after_len - before_len;
                if before_len > after_len {
                    array_len = 0;
                }
                let mut array = Vec::new();
                for _ in 0..array_len {
                    if let Some(x) = self.pop() {
                        array.push(x);
                    }
                }
                let id = self.current_array;
                array.reverse();
                self.arrays.insert(id, array);
                self.current_array += 1;
                self.stack.push_front(Data::Array(id));
            }
            Node::Let(name, span) => {
                if let Some(a) = self.pop() {
                    if let Some(scope) = self.namespace.locals.last_mut() {
                        scope.insert(name, Variable(a));
                    } else {
                        self.namespace.globals.insert(name, Variable(a));
                    }
                } else {
                    return Err(RuntimeError::UnboundVariable(
                        span.to_filespan(self.filename.to_string()),
                        name.to_string(),
                    ));
                }
            }
            Node::AsLet(vars, block, _) => {
                let mut locals = HashMap::new();
                for x in vars.into_iter().rev() {
                    if let Some(a) = self.pop() {
                        locals.insert(x.value.as_str(), Variable(a));
                    } else {
                        return Err(RuntimeError::UnboundVariable(
                            x.span.to_filespan(self.filename.to_string()),
                            x.value.to_string(),
                        ));
                    }
                }
                self.namespace.locals.push(locals);
                self.run_block(block)?;
                self.namespace.locals.pop().unwrap();
            }
            Node::Proc(..) => {}
            Node::Def(..) => {}
            Node::Import(..) => {}
        }
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), RuntimeError> {
        self.prepopulate_namespace()?;
        for n in self.input {
            self.run_node(n)?;
        }
        // println!("{:?}", self.arrays);
        // println!("{:?}", &self.memory_ptr);
        // println!("{:?}", String::from_utf8(self.string_buffer[0..140].iter().map(|x| *x).collect::<Vec<u8>>()));
        Ok(())
    }

    fn run_block(&mut self, b: &'a Vec<Node>) -> Result<(), RuntimeError> {
        for n in b {
            self.run_node(n)?;
            // This checking is (also) made here because a Node can have blocks.
            // If I do run_node() on a block node, it will execute the entire thing
            // even if there is a return, break or continue inside it. If this check is
            // made, it ensures that the block is not further executed after break, continue
            // or return and lets the outer runtime scope handle the condition properly.
            if self.loop_continue || self.loop_break || self.proc_return {
                break;
            }
        }
        Ok(())
    }

    fn push_int(&mut self, n: i64) {
        self.stack.push_front(Data::Int(n));
    }
    
    fn push_float(&mut self, n: f64) {
        self.stack.push_front(Data::Float(n));
    }

    // Accepts a sized Pile string, reads bytes in memory and returns the string read.
    fn read_string(&self, ptr: usize, size: usize) -> &str {
        let slice = &self.string_buffer[ptr..(ptr+size)];
        return std::str::from_utf8(slice).unwrap();
    }
    
    // Stores bytes in memory and returns the resultant sized string that was written
    fn store_string(&mut self, bytes: &[u8]) -> (usize, usize) {
        let size = bytes.len();
        let ptr = self.string_ptr;
        if ptr + size > STR_CAPACITY {
            eprintln!("string buffer size = {}", STR_CAPACITY);
            eprintln!("current pointer = {}", self.string_ptr);
            eprintln!("tried to write {:?}", bytes);
            eprintln!("  size = {}", bytes.len());
            eprintln!("  current pointer + size = {}", self.string_ptr + bytes.len());
            fatal("string buffer overflow");
        }
        self.string_buffer[ptr..ptr + size].copy_from_slice(bytes);
        self.string_ptr += size;
        (ptr, size)
    }

    // Allocate string takes raw bytes, allocates it and then pushes it
    fn push_string(&mut self, bytes: &[u8]) {
        let (ptr, size) = self.store_string(bytes);
        self.stack.push_front(Data::String(ptr, size));
    }

    // Allocates an array in the runtime memory and returns its corresponding ID
    fn alloc_array(&mut self, arr: Vec<Data>) -> u32 {
        let id  = self.current_array;
        self.arrays.insert(id, arr);
        self.current_array += 1;
        return id;
    }

    // Loads array from runtime memory and returns it
    fn load_array(&mut self, id: u32) -> &Vec<Data> {
        return self.arrays.get(&id).unwrap();
    }

    // Allocates arr and also pushes it on the working stack
    fn push_array(&mut self, arr: Vec<Data>) {
        let id = self.alloc_array(arr);
        self.stack.push_front(Data::Array(id));
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
