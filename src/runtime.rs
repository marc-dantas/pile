use std::{collections::HashMap, hash::Hash};

use crate::{compiler::{Addr, Id, Instr, Op, Value}, lexer::FileSpan};

#[derive(Debug, Clone)]
pub enum RuntimeError {
    StackUnderflow(FileSpan, String, usize), // when there's too few data on the stack to perform operation
    UnexpectedType(FileSpan, String, String, String), // when there's an operation tries to operate with an invalid datatype
    InvalidSymbol(FileSpan, String), // used when a word isn't defined
    EmptyDefinition(FileSpan, String), // when a definition is empty
    ProcRedefinition(FileSpan, String), // when a procedure name is already taken
    ArrayOutOfBounds(FileSpan, i64, usize), // when tries to index array at invalid index
    StringOutOfBounds(FileSpan, i64, usize), // when tries to index string at invalid index
    DivisionByZero(FileSpan), // when tries to divide by zero
}

pub struct Executor {
    pub program: Vec<Instr>,
    span: FileSpan,

    stack: Vec<Value>, // Normal data stack

    call_stack: Vec<Addr>,

    strings: HashMap<Id, String>,
    strings_intern_pool: HashMap<String, Id>,
    string_id: Id,

    namespace: Vec<HashMap<String, Value>>,
    definitions: HashMap<String, Value>,
}

fn is_truthy(value: Value) -> bool {
    match value {
        Value::Nil | Value::Bool(false) => false,
        _ => true,
    }
}

impl Executor {
    pub fn new(program: Vec<Instr>) -> Self {
        Self {
            program,
            span: FileSpan::default(),
            stack: Vec::new(),
            strings: HashMap::new(),
            strings_intern_pool: HashMap::new(),
            string_id: 0,
            namespace: Vec::new(),
            call_stack: Vec::new(),
            definitions: HashMap::new(),
        }
    }

    fn run_op(&mut self, op: Op) -> Result<(), RuntimeError> {
        match op {
            Op::Add => {
                let b = self.stack.pop().ok_or(RuntimeError::StackUnderflow(self.span.clone(), "+".to_string(), 2))?;
                let a = self.stack.pop().ok_or(RuntimeError::StackUnderflow(self.span.clone(), "+".to_string(), 2))?;
                match (a, b) {
                    (Value::Int(x), Value::Int(y)) => self.stack.push(Value::Int(x.overflowing_add(y).0)),
                    (Value::Float(x), Value::Float(y)) => self.stack.push(Value::Float(x + y)),
                    _ => return Err(RuntimeError::UnexpectedType(self.span.clone(), "+".to_string(), "two numeric values".to_string(), format!("{:?} and {:?}", a, b))),
                }
                Ok(())
            }
            Op::Sub => {
                let b = self.stack.pop().ok_or(RuntimeError::StackUnderflow(self.span.clone(), "-".to_string(), 2))?;
                let a = self.stack.pop().ok_or(RuntimeError::StackUnderflow(self.span.clone(), "-".to_string(), 2))?;
                match (a, b) {
                    (Value::Int(x), Value::Int(y)) => self.stack.push(Value::Int(x.overflowing_sub(y).0)),
                    (Value::Float(x), Value::Float(y)) => self.stack.push(Value::Float(x - y)),
                    _ => return Err(RuntimeError::UnexpectedType(self.span.clone(), "-".to_string(), "two numeric values".to_string(), format!("{:?} and {:?}", a, b))),
                }
                Ok(())
            }
            Op::Mul => {
                let b = self.stack.pop().ok_or(RuntimeError::StackUnderflow(self.span.clone(), "*".to_string(), 2))?;
                let a = self.stack.pop().ok_or(RuntimeError::StackUnderflow(self.span.clone(), "*".to_string(), 2))?;
                match (a, b) {
                    (Value::Int(x), Value::Int(y)) => self.stack.push(Value::Int(x.overflowing_mul(y).0)),
                    (Value::Float(x), Value::Float(y)) => self.stack.push(Value::Float(x * y)),
                    _ => return Err(RuntimeError::UnexpectedType(self.span.clone(), "*".to_string(), "two numeric values".to_string(), format!("{:?} and {:?}", a, b))),
                }
                Ok(())
            }
            Op::Div => {
                let b = self.stack.pop().ok_or(RuntimeError::StackUnderflow(self.span.clone(), "/".to_string(), 2))?;
                let a = self.stack.pop().ok_or(RuntimeError::StackUnderflow(self.span.clone(), "/".to_string(), 2))?;
                match (a, b) {
                    (Value::Int(x), Value::Int(y)) => {
                        if y == 0 {
                            return Err(RuntimeError::DivisionByZero(self.span.clone()));
                        }
                        self.stack.push(Value::Int(x.overflowing_div(y).0));
                    }
                    (Value::Float(x), Value::Float(y)) => {
                        if y == 0.0 {
                            return Err(RuntimeError::DivisionByZero(self.span.clone()));
                        }
                        self.stack.push(Value::Float(x / y));
                    }
                    _ => return Err(RuntimeError::UnexpectedType(self.span.clone(), "/".to_string(), "two numeric values".to_string(), format!("{:?} and {:?}", a, b))),
                }
                Ok(())
            }
            Op::Mod => {
                let b = self.stack.pop().ok_or(RuntimeError::StackUnderflow(self.span.clone(), "%".to_string(), 2))?;
                let a = self.stack.pop().ok_or(RuntimeError::StackUnderflow(self.span.clone(), "%".to_string(), 2))?;
                match (a, b) {
                    (Value::Int(x), Value::Int(y)) => {
                        if y == 0 {
                            return Err(RuntimeError::DivisionByZero(self.span.clone()));
                        }
                        self.stack.push(Value::Int(x % y));
                    }
                    (Value::Float(x), Value::Float(y)) => {
                        if y == 0.0 {
                            return Err(RuntimeError::DivisionByZero(self.span.clone()));
                        }
                        self.stack.push(Value::Float(x % y));
                    }
                    _ => return Err(RuntimeError::UnexpectedType(self.span.clone(), "%".to_string(), "two numeric values".to_string(), format!("{:?} and {:?}", a, b))),
                }
                Ok(())
            }
            Op::Trace => {
                let a = self.stack.last().ok_or(RuntimeError::StackUnderflow(self.span.clone(), "trace".to_string(), 1))?;
                println!("{:?}", a);
                Ok(())
            }
            _ => todo!(),
        }
    }

    pub fn run(mut self) -> Result<(), RuntimeError> {
        let mut pc = 0;
        while pc < self.program.len() {
            match &self.program[pc] {
                Instr::Jump(addr) => {
                    pc = *addr;
                    continue;
                }
                Instr::JumpIfNot(addr) => {
                    if let Some(x) = self.stack.pop() {
                        if !is_truthy(x) {
                            pc = *addr;
                            continue;
                        }
                    } else {
                        return Err(RuntimeError::StackUnderflow(self.span.clone(), "if".to_string(), 1));
                    }
                }
                Instr::Push(value) => {
                    self.stack.push(*value);
                }
                Instr::Drop => {
                    if let None = self.stack.pop() {
                        return Err(RuntimeError::StackUnderflow(self.span.clone(), "drop".to_string(), 1));
                    }
                }
                Instr::Duplicate => {
                    if let Some(value) = self.stack.last() {
                        self.stack.push(*value);
                    } else {
                        return Err(RuntimeError::StackUnderflow(self.span.clone(), "dup".to_string(), 1));
                    }
                }
                Instr::Swap => {
                    if let (Some(a), Some(b)) = (self.stack.pop(), self.stack.pop()) {
                        self.stack.push(a);
                        self.stack.push(b);
                    } else {
                        return Err(RuntimeError::StackUnderflow(self.span.clone(), "swap".to_string(), 2));
                    }
                }
                Instr::Over => {
                    if let (Some(a), Some(b)) = (self.stack.pop(), self.stack.pop()) {
                        self.stack.push(b);
                        self.stack.push(a);
                        self.stack.push(b);
                    } else {
                        return Err(RuntimeError::StackUnderflow(self.span.clone(), "over".to_string(), 2));
                    }
                }
                Instr::Rotate => {
                    if let (Some(a), Some(b), Some(c)) = (self.stack.pop(), self.stack.pop(), self.stack.pop()) {
                        self.stack.push(b);
                        self.stack.push(a);
                        self.stack.push(c);
                    } else {
                        return Err(RuntimeError::StackUnderflow(self.span.clone(), "rot".to_string(), 3));
                    }
                }
                Instr::ExecOp(op) => {
                    self.run_op(*op)?;
                }
                Instr::BeginScope => {
                    self.namespace.push(HashMap::new());
                }
                Instr::EndScope => {
                    self.namespace.pop();
                }
                Instr::Call(addr) => {
                    self.call_stack.push(pc + 1);
                    pc = *addr;
                    continue;
                }
                Instr::Return => {
                    if let Some(return_addr) = self.call_stack.pop() {
                        pc = return_addr;
                        continue;
                    }
                    unreachable!("Return without a call stack");
                }
                Instr::SetDefinition(name) => {
                    if let Some(value) = self.stack.pop() {
                        self.definitions.insert(name.clone(), value);
                    } else {
                        return Err(RuntimeError::EmptyDefinition(self.span.clone(), format!("{}", name)));
                    }
                }
                Instr::SetVariable(name) => {
                    if let Some(scope) = self.namespace.last_mut() {
                        if let Some(value) = self.stack.pop() {
                            scope.insert(name.clone(), value);
                        } else {
                            return Err(RuntimeError::StackUnderflow(self.span.clone(), format!("{}", name), 1));
                        }
                    }
                }
                Instr::PushBinding(name) => {
                    // check for definitions first
                    if let Some(value) = self.definitions.get(name) {
                        self.stack.push(*value);
                    } else {
                        let mut ok = false;
                        for scope in self.namespace.iter().rev() {
                            if let Some(value) = scope.get(name) {
                                self.stack.push(*value);
                                ok = true;
                                break;
                            }
                        }
                        if !ok {
                            return Err(RuntimeError::InvalidSymbol(self.span.clone(), name.clone()));
                        }
                    }
                }
                Instr::SetSpan(span)  => {
                    // Set the current span for error reporting
                    self.span = span.clone();
                }
                Instr::PushString(value) => {
                    // Create a new string and push it onto the stack
                    if let Some(id) = self.strings_intern_pool.get(value) {
                        self.stack.push(Value::String(*id));
                    } else {
                        let value = value.clone();
                        let id = self.string_id;
                        self.strings_intern_pool.insert(value.clone(), id);
                        self.strings.insert(id, value);
                        self.stack.push(Value::String(id));
                        self.string_id += 1;
                    }
                }
                Instr::ExecBuiltin(..) => todo!()
            }
            pc += 1;
        }
        Ok(())
    }
}