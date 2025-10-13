use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use crate::core::try_parse_from_file;

use crate::{lexer::{FileSpan, Token}, parser::{Node, OpKind}};

#[derive(Debug, Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum Builtin {
    print,
    println,
    eprint,
    eprintln,
    open,
    write,
    read,
    input,
    inputln,
    exit,
    chr,
    ord,
    len,
    typeof_,
    toint,
    tofloat,
    tostring,
    tobool,
}

#[derive(Debug, Clone, Copy)]
pub enum Op {
    // Math
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Exp,
    
    // Relational
    Gt,
    Lt,
    Eq,
    Ge,
    Le,
    Ne,

    // Logic
    Shl,
    Shr,
    Bor,
    Band,
    BNot,

    // Language-specific
    IsNil,
    Index,
    AssignAtIndex,
    
    // Other
    Trace,
}

impl Op {
    fn from_opkind(op: OpKind) -> Self {
        match op {
            OpKind::Add => Op::Add,
            OpKind::Sub => Op::Sub,
            OpKind::Mul => Op::Mul,
            OpKind::Div => Op::Div,
            OpKind::Mod => Op::Mod,
            OpKind::Exp => Op::Exp,
            OpKind::Gt => Op::Gt,
            OpKind::Lt => Op::Lt,
            OpKind::Eq => Op::Eq,
            OpKind::Ge => Op::Ge,
            OpKind::Le => Op::Le,
            OpKind::Ne => Op::Ne,
            OpKind::Shl => Op::Shl,
            OpKind::Shr => Op::Shr,
            OpKind::Bor => Op::Bor,
            OpKind::Band => Op::Band,
            OpKind::BNot => Op::BNot,
            OpKind::IsNil => Op::IsNil,
            OpKind::SeqIndex => Op::Index,
            OpKind::SeqAssignAtIndex => Op::AssignAtIndex,
            OpKind::Trace => Op::Trace,
            _ => unreachable!("bug in the compiler, the operation {:?} should be implemented manually inside the compiler or added here.", op),
        }
    }
}

impl std::fmt::Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Op::Add => write!(f, "'+'"),
            Op::Sub => write!(f, "'-'"),
            Op::Mul => write!(f, "'*'"),
            Op::Div => write!(f, "'/'"),
            Op::Mod => write!(f, "'%'"),
            Op::Exp => write!(f, "'**'"),
            Op::Gt => write!(f, "'>'"),
            Op::Lt => write!(f, "'<'"),
            Op::Eq => write!(f, "'=='"),
            Op::Ge => write!(f, "'>='"),
            Op::Le => write!(f, "'<='"),
            Op::Ne => write!(f, "'!='"),
            Op::Shl => write!(f, "'<<'"),
            Op::Shr => write!(f, "'>>'"),
            Op::Bor => write!(f, "'|'"),
            Op::Band => write!(f, "'&'"),
            Op::BNot => write!(f, "'~'"),
            Op::IsNil => write!(f, "'?'"),
            Op::Index => write!(f, "'@'"),
            Op::AssignAtIndex => write!(f, "'!'"),
            Op::Trace => write!(f, "'trace'"),
        }
    }
}

pub type Addr = usize;
pub type Id = usize;

#[derive(Debug)]
pub enum FileLike {
    File(File),
    Stdin(std::io::Stdin),
    Stdout(std::io::Stdout),
    Stderr(std::io::Stderr),
}

impl FileLike {
    pub fn read(&mut self) -> Option<(String, std::io::Result<usize>)> {
        let mut value = None;
        let mut buf: String = String::new();
        match self {
            FileLike::File(f) => {
                let a = f.read_to_string(&mut buf);
                value = Some((buf, a));
            },
            FileLike::Stdin(f) => {
                let a = f.read_to_string(&mut buf);
                value = Some((buf, a));
            },
            FileLike::Stdout(..) => {},
            FileLike::Stderr(..) => {},
        };
        value
    }

    pub fn write(&mut self, buf: &String) -> Option<std::io::Result<usize>> {
        let mut value = None;
        match self {
            FileLike::File(f) => {
                value = Some(f.write(buf.as_bytes()));
            },
            FileLike::Stdin(f) => {},
            FileLike::Stdout(f) => {
                value = Some(f.write(buf.as_bytes()));
            },
            FileLike::Stderr(f) => {
                value = Some(f.write(buf.as_bytes()));
            },
        };
        value
    }
}

#[derive(Debug)]
pub enum Data {
    File(FileLike),
}

impl std::fmt::Display for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Data::File(..) => write!(f, "file"),
        }
    }
}


#[derive(Debug, Clone, Copy)]
pub enum Value {
    Nil,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(Id),
    Array(Id),
    Data(Id),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Bool(b) => write!(f, "bool {}", b),
            Value::Int(i) => write!(f, "int {}", i),
            Value::Float(fl) => write!(f, "float {}", fl),
            Value::String(id) => write!(f, "string(0x{:0>16X})", id),
            Value::Array(id) => write!(f, "array(0x{:0>16X})", id),
            Value::Data(id) => write!(f, "data(0x{:0>16X})", id),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Instr {
    ExecBuiltin(Builtin),
    Jump(Addr),
    JumpIfNot(Addr),
    ExecOp(Op),
    Push(Value),
    BeginScope,
    EndScope,
    SetVariable(String),
    SetDefinition(String),
    PushBinding(String),
    PushString(String),
    BeginArray,
    EndArray,
    Return,
    Call(Addr),
    Swap,
    Over,
    Duplicate,
    Drop,
    Rotate,
    SetSpan(usize),
}

impl std::fmt::Display for Instr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Instr::ExecBuiltin(builtin) => write!(f, "builtin {:?}", builtin),
            Instr::Jump(addr) => write!(f, "jump 0x{:0>16X}", addr),
            Instr::JumpIfNot(addr) => write!(f, "jumpifnot 0x{:0>16X}", addr),
            Instr::ExecOp(op) => write!(f, "op {}", op),
            Instr::Push(value) => write!(f, "push {}", value),
            Instr::BeginScope => write!(f, "beginscope"),
            Instr::EndScope => write!(f, "endscope"),
            Instr::SetVariable(name) => write!(f, "set $'{}'", name),
            Instr::SetDefinition(name) => write!(f, "set $'{}'", name),
            Instr::PushBinding(name) => write!(f, "push $'{}'", name),
            Instr::PushString(string) => write!(f, "push string \"{}\"", string),
            Instr::BeginArray => write!(f, "beginarray"),
            Instr::EndArray => write!(f, "endarray"),
            Instr::Return => write!(f, "return"),
            Instr::Call(addr) => write!(f, "call 0x{:0>16X}", addr),
            Instr::Swap => write!(f, "swap"),
            Instr::Over => write!(f, "over"),
            Instr::Duplicate => write!(f, "dup"),
            Instr::Drop => write!(f, "drop"),
            Instr::Rotate => write!(f, "rot"),
            Instr::SetSpan(span) => write!(f, "setspan {}", span),
        }
    }
}

pub struct Compiler {
    pub filename: String,
    spans: Vec<FileSpan>,
    instructions: Vec<Instr>,
    procs: HashMap<String, Addr>,
    loop_stack: Vec<(Addr, Vec<Addr>)>
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            filename: String::new(),
            procs: HashMap::new(),
            spans: Vec::new(),
            instructions: Vec::new(),
            loop_stack: Vec::new(),
        }
    }

    pub fn compile(mut self, input: Vec<Node>, filename: String) -> (Vec<Instr>, Vec<FileSpan>) {
        self.filename = filename;
        self.compile_block(input, true);
        (self.instructions, self.spans)
    }

    fn add_span(&mut self, fs: FileSpan) -> usize {
        let id = self.spans.len();
        self.spans.push(fs);
        return id;
    }

    fn compile_block(&mut self, block: Vec<Node>, scoped: bool) {
        // IMPORTANT TODO: Add scope for ifs and loops block.
        //                 I removed because it would fuck up the scoping when doing recursion
        //                 inside other scoped blocks because it "forgets" to close the scope
        //                 that was opened because of ifs and loops inside the proc.
        //                 Probable solution: Add more information about the origin of the scope and
        //                                    make return instruction delete all the scopes that were created
        //                                    inside the proc.
        if scoped { self.instructions.push(Instr::BeginScope); }

        for stmt in block.into_iter() {
            match stmt {
                Node::Import(name, _span) => {
                    let prev_filename = self.filename.to_owned();
                    self.filename = name.clone();
                    self.compile_block(try_parse_from_file(&name), true);
                    self.filename = prev_filename;
                }
                Node::Proc(name, block, span) => {
                    let span_id = self.add_span(span.to_filespan(self.filename.clone()));
                    // NOTE: This SetSpan instruction is not really necessary,
                    // but it will eventually be useful for a future step debugger.
                    self.instructions.push(Instr::SetSpan(span_id));
                    
                    let backpatch = self.instructions.len();
                    self.instructions.push(Instr::Jump(0));
                    let proc_addr = self.instructions.len();
                    self.procs.insert(name, proc_addr);
                    self.compile_block(block, true);
                    self.instructions.push(Instr::Return);
                    self.instructions[backpatch] = Instr::Jump(self.instructions.len());
                }
                Node::If(then_block, else_block, span) => {
                    let span_id = self.add_span(span.to_filespan(self.filename.clone()));
                    self.instructions.push(Instr::SetSpan(span_id));
                    
                    let cond_backpatch = self.instructions.len();
                    self.instructions.push(Instr::JumpIfNot(0));
                    
                    self.compile_block(then_block, false);
                    let escape_backpatch = self.instructions.len();
                    self.instructions.push(Instr::Jump(0));
                    let else_addr = self.instructions.len();
                    if let Some(else_block) = else_block {
                        self.compile_block(else_block, false);
                    }
                    let end = self.instructions.len();
                    self.instructions[escape_backpatch] = Instr::Jump(end);
                    self.instructions[cond_backpatch] = Instr::JumpIfNot(else_addr);
                }
                Node::Loop(block, span) => {
                    let span_id = self.add_span(span.to_filespan(self.filename.clone()));
                    self.instructions.push(Instr::SetSpan(span_id));

                    let loop_start = self.instructions.len();
                    self.loop_stack.push((loop_start, Vec::new()));

                    self.compile_block(block, false);

                    self.instructions.push(Instr::Jump(loop_start));

                    let (_, breaks) = self.loop_stack.pop().unwrap();
                    let loop_end = self.instructions.len(); // after the unconditional jump

                    for break_addr in breaks {
                        self.instructions[break_addr] = Instr::Jump(loop_end);
                    }
                }
                Node::Def(name, block, span) => {
                    let span_id = self.add_span(span.to_filespan(self.filename.clone()));
                    self.instructions.push(Instr::SetSpan(span_id));
                    self.compile_block(block, false);
                    self.instructions.push(Instr::SetDefinition(name));
                }
                Node::Array(block, span) => {
                    let span_id = self.add_span(span.to_filespan(self.filename.clone()));
                    self.instructions.push(Instr::SetSpan(span_id));
                    self.instructions.push(Instr::BeginArray);
                    self.compile_block(block, false);
                    self.instructions.push(Instr::EndArray);
                }
                Node::Operation(OpKind::Break, span) => {
                    let span_id = self.add_span(span.to_filespan(self.filename.clone()));
                    if let Some((_, breaks)) = self.loop_stack.last_mut() {
                        self.instructions.push(Instr::SetSpan(span_id));
                        let break_pos = self.instructions.len();
                        self.instructions.push(Instr::Jump(0)); // placeholder
                        breaks.push(break_pos);
                    }
                }
                Node::Operation(OpKind::Continue, span) => {
                    let span_id = self.add_span(span.to_filespan(self.filename.clone()));
                    if let Some((loop_start, _)) = self.loop_stack.last() {
                        self.instructions.push(Instr::SetSpan(span_id));
                        self.instructions.push(Instr::Jump(*loop_start));
                    }
                }
                Node::Operation(OpKind::Return, span) => {
                    let span_id = self.add_span(span.to_filespan(self.filename.clone()));
                    self.instructions.push(Instr::SetSpan(span_id));
                    self.instructions.push(Instr::EndScope);
                    self.instructions.push(Instr::Return);
                }
                Node::Operation(OpKind::True, span) =>  {
                    let span_id = self.add_span(span.to_filespan(self.filename.clone()));
                    self.instructions.push(Instr::SetSpan(span_id));
                    self.instructions.push(Instr::Push(Value::Bool(true)));
                }
                Node::Operation(OpKind::False, span) => {
                    let span_id = self.add_span(span.to_filespan(self.filename.clone()));
                    self.instructions.push(Instr::SetSpan(span_id));
                    self.instructions.push(Instr::Push(Value::Bool(false)));
                }
                Node::Operation(OpKind::Nil, span) =>   {
                    let span_id = self.add_span(span.to_filespan(self.filename.clone()));
                    self.instructions.push(Instr::SetSpan(span_id));
                    self.instructions.push(Instr::Push(Value::Nil));
                }
                Node::Operation(OpKind::Swap, span) => {
                    let span_id = self.add_span(span.to_filespan(self.filename.clone()));
                    self.instructions.push(Instr::SetSpan(span_id));
                    self.instructions.push(Instr::Swap);
                }
                Node::Operation(OpKind::Over, span) => {
                    let span_id = self.add_span(span.to_filespan(self.filename.clone()));
                    self.instructions.push(Instr::SetSpan(span_id));
                    self.instructions.push(Instr::Over);
                }
                Node::Operation(OpKind::Dup, span) => {
                    let span_id = self.add_span(span.to_filespan(self.filename.clone()));
                    self.instructions.push(Instr::SetSpan(span_id));
                    self.instructions.push(Instr::Duplicate);
                }
                Node::Operation(OpKind::Drop, span) => {
                    let span_id = self.add_span(span.to_filespan(self.filename.clone()));
                    self.instructions.push(Instr::SetSpan(span_id));
                    self.instructions.push(Instr::Drop);
                }
                Node::Operation(OpKind::Rot, span) => {
                    let span_id = self.add_span(span.to_filespan(self.filename.clone()));
                    self.instructions.push(Instr::SetSpan(span_id));
                    self.instructions.push(Instr::Rotate);
                }
                Node::Operation(kind, span) => {
                    let span_id = self.add_span(span.to_filespan(self.filename.clone()));
                    self.instructions.push(Instr::SetSpan(span_id));
                    self.instructions.push(Instr::ExecOp(Op::from_opkind(kind)));
                }
                Node::IntLit(value, span) => {
                    let span_id = self.add_span(span.to_filespan(self.filename.clone()));
                    self.instructions.push(Instr::SetSpan(span_id));
                    self.instructions.push(Instr::Push(Value::Int(value)));
                }
                Node::FloatLit(value, span) => {
                    let span_id = self.add_span(span.to_filespan(self.filename.clone()));
                    self.instructions.push(Instr::SetSpan(span_id));
                    self.instructions.push(Instr::Push(Value::Float(value)));
                }
                Node::StringLit(value, span) => {
                    let span_id = self.add_span(span.to_filespan(self.filename.clone()));
                    self.instructions.push(Instr::SetSpan(span_id));
                    self.instructions.push(Instr::PushString(value));
                }
                Node::Let(name, span) => {
                    let span_id = self.add_span(span.to_filespan(self.filename.clone()));
                    self.instructions.push(Instr::SetSpan(span_id));
                    self.instructions.push(Instr::SetVariable(name));
                }
                Node::Symbol(name, span) => {
                    let span_id = self.add_span(span.to_filespan(self.filename.clone()));
                    self.instructions.push(Instr::SetSpan(span_id));
                    if let Some(addr) = self.procs.get(&name) {
                        self.instructions.push(Instr::Call(*addr));
                    } else {
                        match name.as_str() {
                            "print" => self.instructions.push(Instr::ExecBuiltin(Builtin::print)),
                            "println" => self.instructions.push(Instr::ExecBuiltin(Builtin::println)),
                            "eprint" => self.instructions.push(Instr::ExecBuiltin(Builtin::eprint)),
                            "eprintln" => self.instructions.push(Instr::ExecBuiltin(Builtin::eprintln)),
                            "input" => self.instructions.push(Instr::ExecBuiltin(Builtin::input)),
                            "inputln" => self.instructions.push(Instr::ExecBuiltin(Builtin::inputln)),
                            "open" => self.instructions.push(Instr::ExecBuiltin(Builtin::open)),
                            "write" => self.instructions.push(Instr::ExecBuiltin(Builtin::write)),
                            "read" => self.instructions.push(Instr::ExecBuiltin(Builtin::read)),
                            "exit" => self.instructions.push(Instr::ExecBuiltin(Builtin::exit)),
                            "chr" => self.instructions.push(Instr::ExecBuiltin(Builtin::chr)),
                            "ord" => self.instructions.push(Instr::ExecBuiltin(Builtin::ord)),
                            "len" => self.instructions.push(Instr::ExecBuiltin(Builtin::len)),
                            "typeof" => self.instructions.push(Instr::ExecBuiltin(Builtin::typeof_)),
                            "toint" => self.instructions.push(Instr::ExecBuiltin(Builtin::toint)),
                            "tofloat" => self.instructions.push(Instr::ExecBuiltin(Builtin::tofloat)),
                            "tostring" => self.instructions.push(Instr::ExecBuiltin(Builtin::tostring)),
                            "tobool" => self.instructions.push(Instr::ExecBuiltin(Builtin::tobool)),
                            _ => self.instructions.push(Instr::PushBinding(name)),
                        }
                    }
                }
                Node::AsLet(variables, .. ) => {
                    for var in variables.into_iter().rev() {
                        let Token{ value: x, span: var_span, .. } = var;
                        let span_id = self.add_span(var_span.to_filespan(self.filename.clone()));
                        self.instructions.push(Instr::SetSpan(span_id));
                        self.instructions.push(Instr::SetVariable(x));
                    }
                }
                _ => unimplemented!(), // Placeholder for other statement types
            }
        }
        if scoped { self.instructions.push(Instr::EndScope); }
    }
}

