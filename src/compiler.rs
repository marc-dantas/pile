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
    
    // Stack
    Swap,
    Over,
    Dup,
    Drop,
    Rot,
    
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
            OpKind::Swap => Op::Swap,
            OpKind::Over => Op::Over,
            OpKind::Trace => Op::Trace,
            OpKind::Dup => Op::Dup,
            OpKind::Drop => Op::Drop,
            OpKind::Rot => Op::Rot,
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
    NewVariable(String),
    PushVariable(String),
    PushString(String),
    SetSpan(FileSpan),
}

pub struct Compiler {
    input: ProgramTree,
    filename: String,
}

impl Compiler {
    pub fn new(input: ProgramTree, filename: String) -> Self {
        Compiler { input, filename }
    }

    pub fn compile(self) -> Vec<Instr> {
        let mut pc = 0;
        let mut instructions = Vec::new();
        let mut jump_stack: Vec<Addr> = Vec::new();
        let mut call_stack: Vec<Addr> = Vec::new();
        for stmt in self.input.into_iter() {
            let pc_here = instructions.len();
            match stmt {
                Node::Operation(OpKind::Break, span) => {
                    instructions.push(Instr::SetSpan(span.to_filespan(self.filename.clone())));
                    instructions.push(Instr::Jump(jump_stack.pop().unwrap()));
                }
                Node::Operation(OpKind::Continue, span) => {
                    instructions.push(Instr::SetSpan(span.to_filespan(self.filename.clone())));
                    instructions.push(Instr::Jump(jump_stack.pop().unwrap() - 1));
                }
                Node::Operation(OpKind::Return, span) => {
                    instructions.push(Instr::SetSpan(span.to_filespan(self.filename.clone())));
                    instructions.push(Instr::Jump(call_stack.pop().unwrap()));
                }
                Node::Operation(OpKind::True, span) =>  {
                    instructions.push(Instr::SetSpan(span.to_filespan(self.filename.clone())));
                    instructions.push(Instr::Push(Value::Bool(true)));
                }
                Node::Operation(OpKind::False, span) => {
                    instructions.push(Instr::SetSpan(span.to_filespan(self.filename.clone())));
                    instructions.push(Instr::Push(Value::Bool(false)));
                }
                Node::Operation(OpKind::Nil, span) =>   {
                    instructions.push(Instr::SetSpan(span.to_filespan(self.filename.clone())));
                    instructions.push(Instr::Push(Value::Nil));
                }
                Node::Operation(kind, span) => {
                    instructions.push(Instr::SetSpan(span.to_filespan(self.filename.clone())));
                    instructions.push(Instr::ExecOp(Op::from_opkind(kind)));
                }
                Node::IntLit(value, span) => {
                    instructions.push(Instr::SetSpan(span.to_filespan(self.filename.clone())));
                    instructions.push(Instr::Push(Value::Int(value)));
                }
                Node::FloatLit(value, span) => {
                    instructions.push(Instr::SetSpan(span.to_filespan(self.filename.clone())));
                    instructions.push(Instr::Push(Value::Float(value)));
                }
                Node::StringLit(value, span) => {
                    instructions.push(Instr::SetSpan(span.to_filespan(self.filename.clone())));
                    instructions.push(Instr::PushString(value));
                }

                _ => unimplemented!(), // Placeholder for other statement types
            }
            pc += instructions.len() - pc_here;
        }
        instructions
    }
}

