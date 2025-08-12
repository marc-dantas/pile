use std::{collections::HashMap, hash::Hash};

use crate::{compiler::{Addr, Builtin, Id, Instr, Op, Value}, lexer::FileSpan};

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
            Op::Exp => {
                let b = self.stack.pop().ok_or(RuntimeError::StackUnderflow(self.span.clone(), "**".to_string(), 2))?;
                let a = self.stack.pop().ok_or(RuntimeError::StackUnderflow(self.span.clone(), "**".to_string(), 2))?;
                match (a, b) {
                    (Value::Int(x), Value::Int(y)) => {
                        if y >= 0 {
                            let y = (y as u64).try_into().unwrap();
                            self.stack.push(Value::Int(x.pow(y)));
                        } else {
                            let y = (-y as u64).try_into().unwrap();
                            self.stack.push(Value::Float(1.0/(x.pow(y) as f64)));
                        }
                    }
                    (Value::Float(x), Value::Float(y)) => {
                        if y >= 0.0 {
                            let y = (y as f64).try_into().unwrap();
                            self.stack.push(Value::Float(x.powf(y)));
                        } else {
                            let y = (-y as f64).try_into().unwrap();
                            self.stack.push(Value::Float(1.0/(x.powf(y) as f64)));
                        }
                    }
                    _ => return Err(RuntimeError::UnexpectedType(self.span.clone(), "**".to_string(), "two numeric values".to_string(), format!("{:?} and {:?}", a, b))),
                }
                Ok(())
            }
            Op::Gt => {
                let b = self.stack.pop().ok_or(RuntimeError::StackUnderflow(self.span.clone(), ">".to_string(), 2))?;
                let a = self.stack.pop().ok_or(RuntimeError::StackUnderflow(self.span.clone(), ">".to_string(), 2))?;
                match (a, b) {
                    (Value::Int(x), Value::Int(y)) => {
                        self.stack.push(Value::Bool(x > y));
                    }
                    (Value::Float(x), Value::Float(y)) => {
                        self.stack.push(Value::Bool(x > y));
                    }
                    _ => return Err(RuntimeError::UnexpectedType(self.span.clone(), ">".to_string(), "two numeric values".to_string(), format!("{:?} and {:?}", a, b))),
                }
                Ok(())
            },
            Op::Lt => {
                let b = self.stack.pop().ok_or(RuntimeError::StackUnderflow(self.span.clone(), "<".to_string(), 2))?;
                let a = self.stack.pop().ok_or(RuntimeError::StackUnderflow(self.span.clone(), "<".to_string(), 2))?;
                match (a, b) {
                    (Value::Int(x), Value::Int(y)) => {
                        self.stack.push(Value::Bool(x < y));
                    }
                    (Value::Float(x), Value::Float(y)) => {
                        self.stack.push(Value::Bool(x < y));
                    }
                    _ => return Err(RuntimeError::UnexpectedType(self.span.clone(), "<".to_string(), "two numeric values".to_string(), format!("{:?} and {:?}", a, b))),
                }
                Ok(())
            },
            Op::Eq => {
                let b = self.stack.pop().ok_or(RuntimeError::StackUnderflow(self.span.clone(), "=".to_string(), 2))?;
                let a = self.stack.pop().ok_or(RuntimeError::StackUnderflow(self.span.clone(), "=".to_string(), 2))?;
                match (a, b) {
                    (Value::Int(x), Value::Int(y)) => {
                        self.stack.push(Value::Bool(x == y));
                    }
                    (Value::Float(x), Value::Float(y)) => {
                        self.stack.push(Value::Bool(x == y));
                    }
                    (Value::String(x), Value::String(y)) => {
                        let x = self.strings.get(&x).unwrap();
                        let y = self.strings.get(&y).unwrap();
                        self.stack.push(Value::Bool(x == y));
                    }
                    _ => return Err(RuntimeError::UnexpectedType(self.span.clone(), "=".to_string(), "two numeric values or strings".to_string(), format!("{:?} and {:?}", a, b))),
                }
                Ok(())
            },
            Op::Ge => {
                let b = self.stack.pop().ok_or(RuntimeError::StackUnderflow(self.span.clone(), ">=".to_string(), 2))?;
                let a = self.stack.pop().ok_or(RuntimeError::StackUnderflow(self.span.clone(), ">=".to_string(), 2))?;
                match (a, b) {
                    (Value::Int(x), Value::Int(y)) => {
                        self.stack.push(Value::Bool(x >= y));
                    }
                    (Value::Float(x), Value::Float(y)) => {
                        self.stack.push(Value::Bool(x >= y));
                    }
                    _ => return Err(RuntimeError::UnexpectedType(self.span.clone(), ">=".to_string(), "two numeric values".to_string(), format!("{:?} and {:?}", a, b))),
                }
                Ok(())
            },
            Op::Le => {
                let b = self.stack.pop().ok_or(RuntimeError::StackUnderflow(self.span.clone(), "<=".to_string(), 2))?;
                let a = self.stack.pop().ok_or(RuntimeError::StackUnderflow(self.span.clone(), "<=".to_string(), 2))?;
                match (a, b) {
                    (Value::Int(x), Value::Int(y)) => {
                        self.stack.push(Value::Bool(x <= y));
                    }
                    (Value::Float(x), Value::Float(y)) => {
                        self.stack.push(Value::Bool(x <= y));
                    }
                    _ => return Err(RuntimeError::UnexpectedType(self.span.clone(), "<=".to_string(), "two numeric values".to_string(), format!("{:?} and {:?}", a, b))),
                }
                Ok(())
            },
            Op::Ne => {
                let b = self.stack.pop().ok_or(RuntimeError::StackUnderflow(self.span.clone(), "!=".to_string(), 2))?;
                let a = self.stack.pop().ok_or(RuntimeError::StackUnderflow(self.span.clone(), "!=".to_string(), 2))?;
                match (a, b) {
                    (Value::Int(x), Value::Int(y)) => {
                        self.stack.push(Value::Bool(x != y));
                    }
                    (Value::Float(x), Value::Float(y)) => {
                        self.stack.push(Value::Bool(x != y));
                    }
                    (Value::String(x), Value::String(y)) => {
                        let x = self.strings.get(&x).unwrap();
                        let y = self.strings.get(&y).unwrap();
                        self.stack.push(Value::Bool(x != y));
                    }
                    _ => return Err(RuntimeError::UnexpectedType(self.span.clone(), "!=".to_string(), "two numeric values or strings".to_string(), format!("{:?} and {:?}", a, b))),
                }
                Ok(())
            },
            Op::Shl => {
                let b = self.stack.pop().ok_or(RuntimeError::StackUnderflow(self.span.clone(), "<<".to_string(), 2))?;
                let a = self.stack.pop().ok_or(RuntimeError::StackUnderflow(self.span.clone(), "<<".to_string(), 2))?;
                match (a, b) {
                    (Value::Int(x), Value::Int(y)) => {
                        self.stack.push(Value::Int(x << y));
                    }
                    _ => return Err(RuntimeError::UnexpectedType(self.span.clone(), "<<".to_string(), "two integers".to_string(), format!("{:?} and {:?}", a, b))),
                }
                Ok(())
            },
            Op::Shr => {
                let b = self.stack.pop().ok_or(RuntimeError::StackUnderflow(self.span.clone(), ">>".to_string(), 2))?;
                let a = self.stack.pop().ok_or(RuntimeError::StackUnderflow(self.span.clone(), ">>".to_string(), 2))?;
                match (a, b) {
                    (Value::Int(x), Value::Int(y)) => {
                        self.stack.push(Value::Int(x >> y));
                    }
                    _ => return Err(RuntimeError::UnexpectedType(self.span.clone(), ">>".to_string(), "two integers".to_string(), format!("{:?} and {:?}", a, b))),
                }
                Ok(())
            },
            Op::Bor => {
                let b = self.stack.pop().ok_or(RuntimeError::StackUnderflow(self.span.clone(), "|".to_string(), 2))?;
                let a = self.stack.pop().ok_or(RuntimeError::StackUnderflow(self.span.clone(), "|".to_string(), 2))?;
                match (a, b) {
                    (Value::Int(x), Value::Int(y)) => {
                        self.stack.push(Value::Int(x | y));
                    }
                    (Value::Bool(x), Value::Bool(y)) => {
                        self.stack.push(Value::Bool(x || y));
                    }
                    _ => return Err(RuntimeError::UnexpectedType(self.span.clone(), "|".to_string(), "two integers or two floats".to_string(), format!("{:?} and {:?}", a, b))),
                }
                Ok(())
            },
            Op::Band => {
                let b = self.stack.pop().ok_or(RuntimeError::StackUnderflow(self.span.clone(), "&".to_string(), 2))?;
                let a = self.stack.pop().ok_or(RuntimeError::StackUnderflow(self.span.clone(), "&".to_string(), 2))?;
                match (a, b) {
                    (Value::Int(x), Value::Int(y)) => {
                        self.stack.push(Value::Int(x & y));
                    }
                    (Value::Bool(x), Value::Bool(y)) => {
                        self.stack.push(Value::Bool(x && y));
                    }
                    _ => return Err(RuntimeError::UnexpectedType(self.span.clone(), "&".to_string(), "two integers or two floats".to_string(), format!("{:?} and {:?}", a, b))),
                }
                Ok(())
            },
            Op::BNot => {
                let a = self.stack.pop().ok_or(RuntimeError::StackUnderflow(self.span.clone(), "~".to_string(), 1))?;
                match a {
                    Value::Int(x) => self.stack.push(Value::Int(!x)),
                    Value::Bool(x) => self.stack.push(Value::Bool(!x)),
                    _ => return Err(RuntimeError::UnexpectedType(self.span.clone(), "~".to_string(), "an integer or a float".to_string(), format!("{:?}", a))),
                }
                Ok(())
            },
            Op::IsNil => {
                let a = self.stack.pop().ok_or(RuntimeError::StackUnderflow(self.span.clone(), "?".to_string(), 1))?;
                match a {
                    Value::Nil => self.stack.push(Value::Bool(true)),
                    _ => self.stack.push(Value::Bool(false)),
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

    pub fn run_builtin(&mut self, builtin: Builtin) -> Result<(), RuntimeError> {
        match builtin {
            Builtin::print => {
                if let Some(value) = self.stack.pop() {
                    match value {
                        Value::Nil => print!("nil"),
                        Value::Bool(b) => print!("{}", b),
                        Value::Int(i) => print!("{}", i),
                        Value::Float(f) => print!("{}", f),
                        Value::String(id) => print!("{}", self.strings.get(&id).unwrap()),
                        Value::Array(_id) => todo!(),
                    }
                } else {
                    return Err(RuntimeError::StackUnderflow(self.span.clone(), "print".to_string(), 1));
                }
            },
            Builtin::println => {
                if let Some(value) = self.stack.pop() {
                    match value {
                        Value::Nil => println!("nil"),
                        Value::Bool(b) => println!("{}", b),
                        Value::Int(i) => println!("{}", i),
                        Value::Float(f) => println!("{}", f),
                        Value::String(id) => println!("{}", self.strings.get(&id).unwrap()),
                        Value::Array(_id) => todo!(),
                    }
                } else {
                    return Err(RuntimeError::StackUnderflow(self.span.clone(), "println".to_string(), 1));
                }
            },
            Builtin::eprint => {
                if let Some(value) = self.stack.pop() {
                    match value {
                        Value::Nil => eprint!("nil"),
                        Value::Bool(b) => eprint!("{}", b),
                        Value::Int(i) => eprint!("{}", i),
                        Value::Float(f) => eprint!("{}", f),
                        Value::String(id) => eprint!("{}", self.strings.get(&id).unwrap()),
                        Value::Array(_id) => todo!(),
                    }
                } else {
                    return Err(RuntimeError::StackUnderflow(self.span.clone(), "eprint".to_string(), 1));
                }
            },
            Builtin::eprintln => {
                if let Some(value) = self.stack.pop() {
                    match value {
                        Value::Nil => eprintln!("nil"),
                        Value::Bool(b) => eprintln!("{}", b),
                        Value::Int(i) => eprintln!("{}", i),
                        Value::Float(f) => eprintln!("{}", f),
                        Value::String(id) => eprintln!("{}", self.strings.get(&id).unwrap()),
                        Value::Array(_id) => todo!(),
                    }
                } else {
                    return Err(RuntimeError::StackUnderflow(self.span.clone(), "eprintln".to_string(), 1));
                }
            },
            Builtin::input => {
                use std::io::Read;
                let mut xs = String::new();
                if let Ok(_) = std::io::stdin().read_to_string(&mut xs) {
                    self.push_string(xs);
                } else {
                    self.stack.push(Value::Nil);
                }
            },
            Builtin::inputln => {
                let mut xs = String::new();
                if let Ok(_) = std::io::stdin().read_line(&mut xs) {
                    self.push_string(xs.trim().to_string());
                } else {
                    self.stack.push(Value::Nil);
                }
            },
            Builtin::exit => {
                if let Some(value) = self.stack.pop() {
                    match value {
                        Value::Int(x) => std::process::exit(x as i32),
                        other => {
                            return Err(RuntimeError::UnexpectedType(self.span.clone(), "exit".to_string(), "an integer".to_string(), format!("{:?}", other)));
                        }
                    }
                } else {
                    return Err(RuntimeError::StackUnderflow(self.span.clone(), "exit".to_string(), 1));
                }
            },
            Builtin::chr => {
                todo!()
            },
            Builtin::ord => {
                todo!()
            },
            Builtin::len => {
                if let Some(value) = self.stack.pop() {
                    match value {
                        Value::String(id) => self.stack.push(Value::Int(self.strings.get(&id).unwrap().len() as i64)),
                        other => {
                            return Err(RuntimeError::UnexpectedType(self.span.clone(), "len".to_string(), "a string or an array".to_string(), format!("{:?}", other)));
                        }
                    }
                } else {
                    return Err(RuntimeError::StackUnderflow(self.span.clone(), "len".to_string(), 1));
                }
            },
        }
        Ok(())
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
                    self.namespace.pop().unwrap();
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
                Instr::ExecBuiltin(builtin) => {
                    self.run_builtin(*builtin)?;
                }
            }
            pc += 1;
        }
        Ok(())
    }

    fn push_string(&mut self, string: String) {
        let id = self.string_id;
        self.strings.insert(id, string);
        self.stack.push(Value::String(id));
        self.string_id += 1;
    }
}