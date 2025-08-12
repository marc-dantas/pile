use std::collections::HashMap;

use crate::{lexer::FileSpan, parser::{Node, OpKind, ProgramTree}};

#[derive(Debug, Clone, Copy)]
pub enum Builtin {
    print,
    println,
    eprint,
    eprintln,
    input,
    inputln,
    exit,
    chr,
    ord,
    len,
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

pub type Addr = usize;
pub type Id = usize;

#[derive(Debug, Clone, Copy)]
pub enum Value {
    Nil,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(Id),
    Array(Id),
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
    Return,
    Call(Addr),
    Swap,
    Over,
    Duplicate,
    Drop,
    Rotate,
    SetSpan(FileSpan),
}

pub struct Compiler {
    pub filename: String,
    instructions: Vec<Instr>,
    procs: HashMap<String, Addr>,
    loop_stack: Vec<(Addr, Vec<Addr>)>
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            filename: String::new(),
            procs: HashMap::new(),
            instructions: Vec::new(),
            loop_stack: Vec::new(),
        }
    }

    pub fn compile(mut self, input: Vec<Node>, filename: String) -> Vec<Instr> {
        self.filename = filename;
        self.compile_block(input, true);
        self.instructions
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
                Node::Proc(name, block, span) => {
                    // NOTE: This SetSpan instruction is not really necessary,
                    // but it will eventually be useful for a future step debugger.
                    self.instructions.push(Instr::SetSpan(span.to_filespan(self.filename.clone())));
                    
                    let backpatch = self.instructions.len();
                    self.instructions.push(Instr::Jump(0));
                    let proc_addr = self.instructions.len();
                    self.procs.insert(name, proc_addr);
                    self.compile_block(block, true);
                    self.instructions.push(Instr::Return);
                    self.instructions[backpatch] = Instr::Jump(self.instructions.len());
                }
                Node::If(then_block, else_block, span) => {
                    self.instructions.push(Instr::SetSpan(span.to_filespan(self.filename.clone())));
                    
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
                    self.instructions.push(Instr::SetSpan(span.to_filespan(self.filename.clone())));

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
                    self.instructions.push(Instr::SetSpan(span.to_filespan(self.filename.clone())));
                    self.compile_block(block, false);
                    self.instructions.push(Instr::SetDefinition(name));
                }
                Node::Operation(OpKind::Break, span) => {
                    if let Some((_, breaks)) = self.loop_stack.last_mut() {
                        self.instructions.push(Instr::SetSpan(span.to_filespan(self.filename.clone())));
                        let break_pos = self.instructions.len();
                        self.instructions.push(Instr::Jump(0)); // placeholder
                        breaks.push(break_pos);
                    }
                }
                Node::Operation(OpKind::Continue, span) => {
                    if let Some((loop_start, _)) = self.loop_stack.last() {
                        self.instructions.push(Instr::SetSpan(span.to_filespan(self.filename.clone())));
                        self.instructions.push(Instr::Jump(*loop_start));
                    }
                }
                Node::Operation(OpKind::Return, span) => {
                    self.instructions.push(Instr::SetSpan(span.to_filespan(self.filename.clone())));
                    self.instructions.push(Instr::EndScope);
                    self.instructions.push(Instr::Return);
                }
                Node::Operation(OpKind::True, span) =>  {
                    self.instructions.push(Instr::SetSpan(span.to_filespan(self.filename.clone())));
                    self.instructions.push(Instr::Push(Value::Bool(true)));
                }
                Node::Operation(OpKind::False, span) => {
                    self.instructions.push(Instr::SetSpan(span.to_filespan(self.filename.clone())));
                    self.instructions.push(Instr::Push(Value::Bool(false)));
                }
                Node::Operation(OpKind::Nil, span) =>   {
                    self.instructions.push(Instr::SetSpan(span.to_filespan(self.filename.clone())));
                    self.instructions.push(Instr::Push(Value::Nil));
                }
                Node::Operation(OpKind::Swap, span) => {
                    self.instructions.push(Instr::SetSpan(span.to_filespan(self.filename.clone())));
                    self.instructions.push(Instr::Swap);
                }
                Node::Operation(OpKind::Over, span) => {
                    self.instructions.push(Instr::SetSpan(span.to_filespan(self.filename.clone())));
                    self.instructions.push(Instr::Over);
                }
                Node::Operation(OpKind::Dup, span) => {
                    self.instructions.push(Instr::SetSpan(span.to_filespan(self.filename.clone())));
                    self.instructions.push(Instr::Duplicate);
                }
                Node::Operation(OpKind::Drop, span) => {
                    self.instructions.push(Instr::SetSpan(span.to_filespan(self.filename.clone())));
                    self.instructions.push(Instr::Drop);
                }
                Node::Operation(OpKind::Rot, span) => {
                    self.instructions.push(Instr::SetSpan(span.to_filespan(self.filename.clone())));
                    self.instructions.push(Instr::Rotate);
                }
                Node::Operation(kind, span) => {
                    self.instructions.push(Instr::SetSpan(span.to_filespan(self.filename.clone())));
                    self.instructions.push(Instr::ExecOp(Op::from_opkind(kind)));
                }
                Node::IntLit(value, span) => {
                    self.instructions.push(Instr::SetSpan(span.to_filespan(self.filename.clone())));
                    self.instructions.push(Instr::Push(Value::Int(value)));
                }
                Node::FloatLit(value, span) => {
                    self.instructions.push(Instr::SetSpan(span.to_filespan(self.filename.clone())));
                    self.instructions.push(Instr::Push(Value::Float(value)));
                }
                Node::StringLit(value, span) => {
                    self.instructions.push(Instr::SetSpan(span.to_filespan(self.filename.clone())));
                    self.instructions.push(Instr::PushString(value));
                }
                Node::Let(name, span) => {
                    self.instructions.push(Instr::SetSpan(span.to_filespan(self.filename.clone())));
                    self.instructions.push(Instr::SetVariable(name));
                }
                Node::Symbol(name, span) => {
                    self.instructions.push(Instr::SetSpan(span.to_filespan(self.filename.clone())));
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
                            "exit" => self.instructions.push(Instr::ExecBuiltin(Builtin::exit)),
                            "chr" => self.instructions.push(Instr::ExecBuiltin(Builtin::chr)),
                            "ord" => self.instructions.push(Instr::ExecBuiltin(Builtin::ord)),
                            "len" => self.instructions.push(Instr::ExecBuiltin(Builtin::len)),
                            _ => self.instructions.push(Instr::PushBinding(name)),
                        }
                    }
                }
                _ => unimplemented!(), // Placeholder for other statement types
            }
        }
        if scoped { self.instructions.push(Instr::EndScope); }
    }
}

