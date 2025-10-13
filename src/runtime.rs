
use std::{collections::HashMap, fs::{File, OpenOptions}, io::{stdout, BufReader, Read, Write}, os::fd::{AsFd, AsRawFd}};
use crate::{compiler::{Addr, Builtin, Data, FileLike, Id, Instr, Op, Value}, lexer::FileSpan};

#[derive(Debug, Clone)]
pub enum RuntimeError {
    StackUnderflow(FileSpan, String, usize), // when there's too few data on the stack to perform operation
    UnexpectedType(FileSpan, String, String, String), // when there's an operation tries to operate with an invalid datatype
    InvalidSymbol(FileSpan, String), // used when a word isn't defined
    EmptyDefinition(FileSpan, String), // when a definition is empty
    ArrayOutOfBounds(FileSpan, i64, usize), // when tries to index array at invalid index
    StringOutOfBounds(FileSpan, i64, usize), // when tries to index string at invalid index
    DivisionByZero(FileSpan), // when tries to divide by zero
    Custom(FileSpan, String), // custom error thrown by misc thing
}

pub struct Executor {
    pub program: Vec<Instr>,
    spans: Vec<FileSpan>,
    span: usize,

    stack: Vec<Value>, // Normal data stack

    call_stack: Vec<Addr>,

    strings_intern_pool: HashMap<String, Id>,
    strings: HashMap<Id, String>,
    string_id: Id,

    array_stack: Vec<usize>,
    //               ^---- The length of the data stack at the point of BeginArray,
    //                     so i can subtract and get the len of the array and get all the items
    arrays: HashMap<Id, Vec<Value>>,
    array_id: Id,

    datas: HashMap<Id, Data>,
    datas_id: Id,

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
    pub fn new(program: Vec<Instr>, spans: Vec<FileSpan>) -> Self {
        Self {
            program,
            span: 0,
            spans: spans,
            stack: Vec::new(),
            strings: HashMap::new(),
            strings_intern_pool: HashMap::new(),
            string_id: 0,
            array_stack: Vec::new(),
            arrays: HashMap::new(),
            array_id: 0,
            datas: HashMap::new(),
            datas_id: 0,
            namespace: Vec::new(),
            call_stack: Vec::new(),
            definitions: HashMap::new(),
        }
    }

    fn get_span(&self) -> FileSpan {
        self.spans.get(self.span).unwrap().clone()
    }

    fn run_op(&mut self, op: Op) -> Result<(), RuntimeError> {
        match op {
            Op::Add => {
                let b = self.stack.pop().ok_or_else(|| RuntimeError::StackUnderflow(self.get_span(), "+".to_string(), 2))?;
                let a = self.stack.pop().ok_or_else(|| RuntimeError::StackUnderflow(self.get_span(), "+".to_string(), 2))?;
                match (a, b) {
                    (Value::Int(x), Value::Int(y)) => self.stack.push(Value::Int(x.overflowing_add(y).0)),
                    (Value::Float(x), Value::Float(y)) => self.stack.push(Value::Float(x + y)),
                    _ => return Err(RuntimeError::UnexpectedType(self.get_span(), "+".to_string(), "two numeric values".to_string(), format!("{} and {}", a, b))),
                }
                Ok(())
            }
            Op::Sub => {
                let b = self.stack.pop().ok_or_else(|| RuntimeError::StackUnderflow(self.get_span(), "-".to_string(), 2))?;
                let a = self.stack.pop().ok_or_else(|| RuntimeError::StackUnderflow(self.get_span(), "-".to_string(), 2))?;
                match (a, b) {
                    (Value::Int(x), Value::Int(y)) => self.stack.push(Value::Int(x.overflowing_sub(y).0)),
                    (Value::Float(x), Value::Float(y)) => self.stack.push(Value::Float(x - y)),
                    _ => return Err(RuntimeError::UnexpectedType(self.get_span(), "-".to_string(), "two numeric values".to_string(), format!("{} and {}", a, b))),
                }
                Ok(())
            }
            Op::Mul => {
                let b = self.stack.pop().ok_or_else(|| RuntimeError::StackUnderflow(self.get_span(), "*".to_string(), 2))?;
                let a = self.stack.pop().ok_or_else(|| RuntimeError::StackUnderflow(self.get_span(), "*".to_string(), 2))?;
                match (a, b) {
                    (Value::Int(x), Value::Int(y)) => self.stack.push(Value::Int(x.overflowing_mul(y).0)),
                    (Value::Float(x), Value::Float(y)) => self.stack.push(Value::Float(x * y)),
                    _ => return Err(RuntimeError::UnexpectedType(self.get_span(), "*".to_string(), "two numeric values".to_string(), format!("{} and {}", a, b))),
                }
                Ok(())
            }
            Op::Div => {
                let b = self.stack.pop().ok_or_else(|| RuntimeError::StackUnderflow(self.get_span(), "/".to_string(), 2))?;
                let a = self.stack.pop().ok_or_else(|| RuntimeError::StackUnderflow(self.get_span(), "/".to_string(), 2))?;
                match (a, b) {
                    (Value::Int(x), Value::Int(y)) => {
                        if y == 0 {
                            return Err(RuntimeError::DivisionByZero(self.get_span()));
                        }
                        self.stack.push(Value::Int(x.overflowing_div(y).0));
                    }
                    (Value::Float(x), Value::Float(y)) => {
                        if y == 0.0 {
                            return Err(RuntimeError::DivisionByZero(self.get_span()));
                        }
                        self.stack.push(Value::Float(x / y));
                    }
                    _ => return Err(RuntimeError::UnexpectedType(self.get_span(), "/".to_string(), "two numeric values".to_string(), format!("{} and {}", a, b))),
                }
                Ok(())
            }
            Op::Mod => {
                let b = self.stack.pop().ok_or_else(|| RuntimeError::StackUnderflow(self.get_span(), "%".to_string(), 2))?;
                let a = self.stack.pop().ok_or_else(|| RuntimeError::StackUnderflow(self.get_span(), "%".to_string(), 2))?;
                match (a, b) {
                    (Value::Int(x), Value::Int(y)) => {
                        if y == 0 {
                            return Err(RuntimeError::DivisionByZero(self.get_span()));
                        }
                        self.stack.push(Value::Int(x % y));
                    }
                    (Value::Float(x), Value::Float(y)) => {
                        if y == 0.0 {
                            return Err(RuntimeError::DivisionByZero(self.get_span()));
                        }
                        self.stack.push(Value::Float(x % y));
                    }
                    _ => return Err(RuntimeError::UnexpectedType(self.get_span(), "%".to_string(), "two numeric values".to_string(), format!("{} and {}", a, b))),
                }
                Ok(())
            }
            Op::Exp => {
                let b = self.stack.pop().ok_or_else(|| RuntimeError::StackUnderflow(self.get_span(), "**".to_string(), 2))?;
                let a = self.stack.pop().ok_or_else(|| RuntimeError::StackUnderflow(self.get_span(), "**".to_string(), 2))?;
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
                    _ => return Err(RuntimeError::UnexpectedType(self.get_span(), "**".to_string(), "two numeric values".to_string(), format!("{} and {}", a, b))),
                }
                Ok(())
            }
            Op::Gt => {
                let b = self.stack.pop().ok_or_else(|| RuntimeError::StackUnderflow(self.get_span(), ">".to_string(), 2))?;
                let a = self.stack.pop().ok_or_else(|| RuntimeError::StackUnderflow(self.get_span(), ">".to_string(), 2))?;
                match (a, b) {
                    (Value::Int(x), Value::Int(y)) => {
                        self.stack.push(Value::Bool(x > y));
                    }
                    (Value::Float(x), Value::Float(y)) => {
                        self.stack.push(Value::Bool(x > y));
                    }
                    _ => return Err(RuntimeError::UnexpectedType(self.get_span(), ">".to_string(), "two numeric values".to_string(), format!("{} and {}", a, b))),
                }
                Ok(())
            },
            Op::Lt => {
                let b = self.stack.pop().ok_or_else(|| RuntimeError::StackUnderflow(self.get_span(), "<".to_string(), 2))?;
                let a = self.stack.pop().ok_or_else(|| RuntimeError::StackUnderflow(self.get_span(), "<".to_string(), 2))?;
                match (a, b) {
                    (Value::Int(x), Value::Int(y)) => {
                        self.stack.push(Value::Bool(x < y));
                    }
                    (Value::Float(x), Value::Float(y)) => {
                        self.stack.push(Value::Bool(x < y));
                    }
                    _ => return Err(RuntimeError::UnexpectedType(self.get_span(), "<".to_string(), "two numeric values".to_string(), format!("{} and {}", a, b))),
                }
                Ok(())
            },
            Op::Eq => {
                let b = self.stack.pop().ok_or_else(|| RuntimeError::StackUnderflow(self.get_span(), "=".to_string(), 2))?;
                let a = self.stack.pop().ok_or_else(|| RuntimeError::StackUnderflow(self.get_span(), "=".to_string(), 2))?;
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
                    _ => return Err(RuntimeError::UnexpectedType(self.get_span(), "=".to_string(), "two numeric values or strings".to_string(), format!("{} and {}", a, b))),
                }
                Ok(())
            },
            Op::Ge => {
                let b = self.stack.pop().ok_or_else(|| RuntimeError::StackUnderflow(self.get_span(), ">=".to_string(), 2))?;
                let a = self.stack.pop().ok_or_else(|| RuntimeError::StackUnderflow(self.get_span(), ">=".to_string(), 2))?;
                match (a, b) {
                    (Value::Int(x), Value::Int(y)) => {
                        self.stack.push(Value::Bool(x >= y));
                    }
                    (Value::Float(x), Value::Float(y)) => {
                        self.stack.push(Value::Bool(x >= y));
                    }
                    _ => return Err(RuntimeError::UnexpectedType(self.get_span(), ">=".to_string(), "two numeric values".to_string(), format!("{} and {}", a, b))),
                }
                Ok(())
            },
            Op::Le => {
                let b = self.stack.pop().ok_or_else(|| RuntimeError::StackUnderflow(self.get_span(), "<=".to_string(), 2))?;
                let a = self.stack.pop().ok_or_else(|| RuntimeError::StackUnderflow(self.get_span(), "<=".to_string(), 2))?;
                match (a, b) {
                    (Value::Int(x), Value::Int(y)) => {
                        self.stack.push(Value::Bool(x <= y));
                    }
                    (Value::Float(x), Value::Float(y)) => {
                        self.stack.push(Value::Bool(x <= y));
                    }
                    _ => return Err(RuntimeError::UnexpectedType(self.get_span(), "<=".to_string(), "two numeric values".to_string(), format!("{} and {}", a, b))),
                }
                Ok(())
            },
            Op::Ne => {
                let b = self.stack.pop().ok_or_else(|| RuntimeError::StackUnderflow(self.get_span(), "!=".to_string(), 2))?;
                let a = self.stack.pop().ok_or_else(|| RuntimeError::StackUnderflow(self.get_span(), "!=".to_string(), 2))?;
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
                    _ => return Err(RuntimeError::UnexpectedType(self.get_span(), "!=".to_string(), "two numeric values or strings".to_string(), format!("{} and {}", a, b))),
                }
                Ok(())
            },
            Op::Shl => {
                let b = self.stack.pop().ok_or_else(|| RuntimeError::StackUnderflow(self.get_span(), "<<".to_string(), 2))?;
                let a = self.stack.pop().ok_or_else(|| RuntimeError::StackUnderflow(self.get_span(), "<<".to_string(), 2))?;
                match (a, b) {
                    (Value::Int(x), Value::Int(y)) => {
                        self.stack.push(Value::Int(x << y));
                    }
                    _ => return Err(RuntimeError::UnexpectedType(self.get_span(), "<<".to_string(), "two integers".to_string(), format!("{} and {}", a, b))),
                }
                Ok(())
            },
            Op::Shr => {
                let b = self.stack.pop().ok_or_else(|| RuntimeError::StackUnderflow(self.get_span(), ">>".to_string(), 2))?;
                let a = self.stack.pop().ok_or_else(|| RuntimeError::StackUnderflow(self.get_span(), ">>".to_string(), 2))?;
                match (a, b) {
                    (Value::Int(x), Value::Int(y)) => {
                        self.stack.push(Value::Int(x >> y));
                    }
                    _ => return Err(RuntimeError::UnexpectedType(self.get_span(), ">>".to_string(), "two integers".to_string(), format!("{} and {}", a, b))),
                }
                Ok(())
            },
            Op::Bor => {
                let b = self.stack.pop().ok_or_else(|| RuntimeError::StackUnderflow(self.get_span(), "|".to_string(), 2))?;
                let a = self.stack.pop().ok_or_else(|| RuntimeError::StackUnderflow(self.get_span(), "|".to_string(), 2))?;
                match (a, b) {
                    (Value::Int(x), Value::Int(y)) => {
                        self.stack.push(Value::Int(x | y));
                    }
                    (Value::Bool(x), Value::Bool(y)) => {
                        self.stack.push(Value::Bool(x || y));
                    }
                    _ => return Err(RuntimeError::UnexpectedType(self.get_span(), "|".to_string(), "two integers or two floats".to_string(), format!("{} and {}", a, b))),
                }
                Ok(())
            },
            Op::Band => {
                let b = self.stack.pop().ok_or_else(|| RuntimeError::StackUnderflow(self.get_span(), "&".to_string(), 2))?;
                let a = self.stack.pop().ok_or_else(|| RuntimeError::StackUnderflow(self.get_span(), "&".to_string(), 2))?;
                match (a, b) {
                    (Value::Int(x), Value::Int(y)) => {
                        self.stack.push(Value::Int(x & y));
                    }
                    (Value::Bool(x), Value::Bool(y)) => {
                        self.stack.push(Value::Bool(x && y));
                    }
                    _ => return Err(RuntimeError::UnexpectedType(self.get_span(), "&".to_string(), "two integers or two floats".to_string(), format!("{} and {}", a, b))),
                }
                Ok(())
            },
            Op::BNot => {
                let a = self.stack.pop().ok_or_else(|| RuntimeError::StackUnderflow(self.get_span(), "~".to_string(), 1))?;
                match a {
                    Value::Int(x) => self.stack.push(Value::Int(!x)),
                    Value::Bool(x) => self.stack.push(Value::Bool(!x)),
                    _ => return Err(RuntimeError::UnexpectedType(self.get_span(), "~".to_string(), "an integer or a float".to_string(), format!("{}", a))),
                }
                Ok(())
            },
            Op::IsNil => {
                let a = self.stack.pop().ok_or_else(|| RuntimeError::StackUnderflow(self.get_span(), "?".to_string(), 1))?;
                match a {
                    Value::Nil => self.stack.push(Value::Bool(true)),
                    _ => self.stack.push(Value::Bool(false)),
                }
                Ok(())
            }
            Op::Trace => {
                let a = self.stack.last().ok_or_else(|| RuntimeError::StackUnderflow(self.get_span(), "trace".to_string(), 1))?;
                println!("{}", a);
                Ok(())
            }
            Op::Index => {
                let index = self.stack.pop().ok_or_else(|| RuntimeError::StackUnderflow(self.get_span(), "@".to_string(), 2))?;
                let seq = self.stack.pop().ok_or_else(|| RuntimeError::StackUnderflow(self.get_span(), "@".to_string(), 2))?;
                match (seq, index) {
                    (Value::Array(id), Value::Int(i)) => {
                        let array = self.arrays.get(&id).unwrap();
                        if let Some(value) = array.get(i as usize) {
                            self.stack.push(value.clone());
                        } else {
                            return Err(RuntimeError::ArrayOutOfBounds(self.get_span(), i, array.len()));
                        }
                    }
                    (Value::String(id), Value::Int(i)) => {
                        let string = self.strings.get(&id).unwrap();
                        if let Some(value) = string.chars().nth(i as usize) {
                            self.push_string(value.to_string());
                        } else {
                            return Err(RuntimeError::StringOutOfBounds(self.get_span(), i, string.len()));
                        }
                    }
                    _ => return Err(RuntimeError::UnexpectedType(self.get_span(), "@".to_string(), "array/string and an integer".to_string(), format!("{} and {}", seq, index))),
                }
                Ok(())
            }
            Op::AssignAtIndex => {
                let value = self.stack.pop().ok_or_else(|| RuntimeError::StackUnderflow(self.get_span(), "!".to_string(), 3))?;
                let index = self.stack.pop().ok_or_else(|| RuntimeError::StackUnderflow(self.get_span(), "!".to_string(), 3))?;
                let seq = self.stack.pop().ok_or_else(|| RuntimeError::StackUnderflow(self.get_span(), "!".to_string(), 3))?;
                match (seq, index, value) {
                    (Value::Array(id), Value::Int(i), value) => {
                        let array = self.arrays.get_mut(&id).unwrap();
                        let array_len = array.len();
                        if i as usize >= array_len {
                            return Err(RuntimeError::ArrayOutOfBounds(self.get_span(), i, array_len));
                        }
                        array[i as usize] = value;
                    }
                    (Value::String(id), Value::Int(i), Value::Int(chrcode)) => {
                        let string = self.strings.get_mut(&id).unwrap();
                        let string_len = string.len();
                        if i as usize >= string_len {
                            return Err(RuntimeError::StringOutOfBounds(self.get_span(), i, string_len));
                        }
                        if let Some(chr) = std::char::from_u32(chrcode as u32) {
                            string.replace_range(i as usize..i as usize + 1, &chr.to_string());
                        } else {
                            string.replace_range(i as usize..i as usize + 1, "\0");
                        }
                    }
                    _ => return Err(RuntimeError::UnexpectedType(self.get_span(), "!".to_string(), "(string, int, int) or (array, int, any)".to_string(), format!("({}, {}, {})", seq, index, value))),
                }
                Ok(())
            }
        }
    }

    pub fn run_builtin(&mut self, builtin: Builtin) -> Result<(), RuntimeError> {
        match builtin {
            Builtin::toint => {
                if let Some(value) = self.stack.pop() {
                    match value {
                        Value::Int(i) => self.stack.push(Value::Int(i)),
                        Value::Float(f) => self.stack.push(Value::Int(f as i64)),
                        Value::String(id) => {
                            let s = self.strings.get(&id).unwrap();
                            if let Ok(i) = s.parse::<i64>() {
                                self.stack.push(Value::Int(i));
                            } else {
                                self.stack.push(Value::Nil);
                            }
                        },
                        Value::Bool(b) => self.stack.push(Value::Int(if b { 1 } else { 0 })),
                        _ => self.stack.push(Value::Nil),
                    }
                } else {
                    return Err(RuntimeError::StackUnderflow(self.get_span(), "toint".to_string(), 1));
                }
            }
            Builtin::tofloat => {
                if let Some(value) = self.stack.pop() {
                    match value {
                        Value::Int(i) => { self.stack.push(Value::Float(i as f64))},
                        Value::Float(f) => self.stack.push(Value::Float(f)),
                        Value::String(id) => {
                            let s = self.strings.get(&id).unwrap();
                            if let Ok(f) = s.parse::<f64>() {
                                self.stack.push(Value::Float(f));
                            } else {
                                self.stack.push(Value::Nil);
                            }
                        },
                        Value::Bool(b) => self.stack.push(Value::Float(if b { 1.0 } else { 0.0 })),
                        _ => self.stack.push(Value::Nil),
                    }
                } else {
                    return Err(RuntimeError::StackUnderflow(self.get_span(), "tofloat".to_string(), 1));
                }
            }
            Builtin::tostring => {
                if let Some(value) = self.stack.pop() {
                    self.push_string(format!("{}", value));
                } else {
                    return Err(RuntimeError::StackUnderflow(self.get_span(), "tostring".to_string(), 1));
                }
            }
            Builtin::tobool => {
                if let Some(value) = self.stack.pop() {
                    match value {
                        Value::Bool(b) => self.stack.push(Value::Bool(b)),
                        Value::Nil => self.stack.push(Value::Bool(false)),
                        Value::Int(i) => self.stack.push(Value::Bool(i != 0)),
                        Value::Float(f) => self.stack.push(Value::Bool(f != 0.0)),
                        Value::String(id) => {
                            let s = self.strings.get(&id).unwrap();
                            self.stack.push(Value::Bool(!s.is_empty()));
                        },
                        Value::Array(id) => {
                            let a = self.arrays.get(&id).unwrap();
                            self.stack.push(Value::Bool(!a.is_empty()));
                        },
                        x => {
                            return Err(RuntimeError::UnexpectedType(self.get_span(), "tobool".to_string(), "bool, nil, int, float, string or array".to_string(), format!("{}", x)));
                        }
                    }
                } else {
                    return Err(RuntimeError::StackUnderflow(self.get_span(), "tobool".to_string(), 1));
                }
            }
            Builtin::typeof_ => {
                let value = self.stack.pop().ok_or_else(|| RuntimeError::StackUnderflow(self.get_span(), "typeof".to_string(), 1))?;
                let type_name = match value {
                    Value::Nil => "nil",
                    Value::Bool(_) => "bool",
                    Value::Int(_) => "int",
                    Value::Float(_) => "float",
                    Value::String(_) => "string",
                    Value::Array(_) => "array",
                    Value::Data(_) => "data",
                };
                self.push_string(type_name.to_string());
            }
            Builtin::open => {
                if let Some(path) = self.stack.pop() {
                    if let Value::String(path) = path {
                        let path = self.strings.get(&path).unwrap();
                        match OpenOptions::new()
                               .write(true)
                               .read(true)
                               .truncate(false)
                               .create(true)
                               .open(path) {
                            Ok(f) => {
                                self.datas.insert(self.datas_id, Data::File(FileLike::File(f)));
                                self.stack.push(Value::Data(self.datas_id));
                                self.datas_id += 1;
                            }
                            Err(e) => {
                                return Err(RuntimeError::Custom(self.get_span(), format!("file error: {}", e.to_string())));
                            }
                        }
                    } else {
                        return Err(RuntimeError::UnexpectedType(self.get_span(), "open".to_string(), "string".to_string(), format!("{}", path)));
                    }
                } else {
                    return Err(RuntimeError::StackUnderflow(self.get_span(), "open".to_string(), 1));
                }
            },
            Builtin::write => {
                if let (Some(buf), Some(file)) = (self.stack.pop(), self.stack.pop()) {
                    if let (Value::Data(file), Value::String(buf)) = (file, buf) {
                        let span = self.get_span();
                        let file = self.datas.get_mut(&file).unwrap();
                        let buf = self.strings.get(&buf).unwrap();
                        if let Data::File(file) = file {
                            match file.write(buf) {
                                Some(std::io::Result::Err(e)) => {
                                    return Err(RuntimeError::Custom(self.get_span(), format!("file error: {}", e.to_string())));
                                }
                                None => {
                                    return Err(RuntimeError::Custom(self.get_span(), format!("file error: not able to write")));
                                }
                                _ => {}
                            }
                        } else {
                            return Err(RuntimeError::UnexpectedType(span, "write".to_string(), "file and a string".to_string(), format!("{}", file)));
                        }
                    } else {
                        return Err(RuntimeError::UnexpectedType(self.get_span(), "write".to_string(), "file and a string".to_string(), format!("{}", file)));
                    }
                } else {
                    return Err(RuntimeError::StackUnderflow(self.get_span(), "write".to_string(), 2));
                }
            },
            Builtin::read => {
                if let Some(file) = self.stack.pop() {
                    if let Value::Data(file) = file {
                        let span = self.get_span();
                        let file = self.datas.get_mut(&file).unwrap();
                        if let Data::File(file) = file {
                            match file.read() {
                                Some((_, std::io::Result::Err(e))) => {
                                    return Err(RuntimeError::Custom(self.get_span(), format!("file error: {}", e.to_string())));
                                }
                                None => {
                                    return Err(RuntimeError::Custom(self.get_span(), format!("file error: not able to read")));
                                }
                                Some((b, std::io::Result::Ok(_))) => {
                                    self.push_string(b);
                                }
                            }
                        } else {
                            return Err(RuntimeError::UnexpectedType(span, "read".to_string(), "file".to_string(), format!("{}", file)));
                        }
                    } else {
                        return Err(RuntimeError::UnexpectedType(self.get_span(), "read".to_string(), "file".to_string(), format!("{}", file)));
                    }
                } else {
                    return Err(RuntimeError::StackUnderflow(self.get_span(), "read".to_string(), 1));
                }
            },
            Builtin::readline => {
                if let Some(file) = self.stack.pop() {
                    if let Value::Data(file) = file {
                        let span = self.get_span();
                        let file = self.datas.get_mut(&file).unwrap();
                        if let Data::File(file) = file {
                            match file.readline() {
                                Some((_, std::io::Result::Err(e))) => {
                                    return Err(RuntimeError::Custom(self.get_span(), format!("file error: {}", e.to_string())));
                                }
                                None => {
                                    return Err(RuntimeError::Custom(self.get_span(), format!("file error: not able to read line")));
                                }
                                Some((b, std::io::Result::Ok(_))) => {
                                    self.push_string(b);
                                }
                            }
                        } else {
                            return Err(RuntimeError::UnexpectedType(span, "readline".to_string(), "file".to_string(), format!("{}", file)));
                        }
                    } else {
                        return Err(RuntimeError::UnexpectedType(self.get_span(), "readline".to_string(), "file".to_string(), format!("{}", file)));
                    }
                } else {
                    return Err(RuntimeError::StackUnderflow(self.get_span(), "readline".to_string(), 1));
                }
            },
            Builtin::exit => {
                if let Some(value) = self.stack.pop() {
                    match value {
                        Value::Int(x) => std::process::exit(x as i32),
                        other => {
                            return Err(RuntimeError::UnexpectedType(self.get_span(), "exit".to_string(), "an integer".to_string(), format!("{}", other)));
                        }
                    }
                } else {
                    return Err(RuntimeError::StackUnderflow(self.get_span(), "exit".to_string(), 1));
                }
            },
            Builtin::chr => {
                if let Some(value) = self.stack.pop() {
                    match value {
                        Value::Int(i) => {
                            if let Some(c) = std::char::from_u32(i as u32) {
                                self.push_string(c.to_string());
                            } else {
                                self.stack.push(Value::Nil);
                            }
                        },
                        other => {
                            return Err(RuntimeError::UnexpectedType(self.get_span(), "chr".to_string(), "an integer".to_string(), format!("{}", other)));
                        }
                    }
                } else {
                    return Err(RuntimeError::StackUnderflow(self.get_span(), "chr".to_string(), 1));
                }
            },
            Builtin::ord => {
                if let Some(value) = self.stack.pop() {
                    match value {
                        Value::String(id) => {
                            let string = self.strings.get(&id).unwrap();
                            if let Some(c) = string.chars().next() {
                                self.stack.push(Value::Int(c as i64));
                            } else {
                                return Err(RuntimeError::UnexpectedType(self.get_span(), "ord".to_string(), "a non-empty string".to_string(), format!("{}", value)));
                            }
                        },
                        other => {
                            return Err(RuntimeError::UnexpectedType(self.get_span(), "ord".to_string(), "a string".to_string(), format!("{}", other)));
                        }
                    }
                } else {
                    return Err(RuntimeError::StackUnderflow(self.get_span(), "ord".to_string(), 1));
                }
            },
            Builtin::len => {
                if let Some(value) = self.stack.pop() {
                    match value {
                        Value::String(id) => self.stack.push(Value::Int(self.strings.get(&id).unwrap().len() as i64)),
                        Value::Array(id) => self.stack.push(Value::Int(self.arrays.get(&id).unwrap().len() as i64)),
                        other => {
                            return Err(RuntimeError::UnexpectedType(self.get_span(), "len".to_string(), "a string or an array".to_string(), format!("{}", other)));
                        }
                    }
                } else {
                    return Err(RuntimeError::StackUnderflow(self.get_span(), "len".to_string(), 1));
                }
            },
        }
        Ok(())
    }

    pub fn new_data(&mut self, data: Data) -> Value {
        let id = self.datas_id;
        self.datas.insert(id, data);
        self.datas_id += 1;
        return Value::Data(id);
    }

    fn header(&mut self) {
        let data = self.new_data(Data::File(FileLike::Stdin(std::io::stdin())));
        self.definitions.insert("STDIN".to_string(), data);
        
        let data = self.new_data(Data::File(FileLike::Stdout(std::io::stdout())));
        self.definitions.insert("STDOUT".to_string(), data);
        
        let data = self.new_data(Data::File(FileLike::Stderr(std::io::stderr())));
        self.definitions.insert("STDERR".to_string(), data);
    }

    pub fn run(mut self) -> Result<(), RuntimeError> {
        // Program Header
        self.header();

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
                        return Err(RuntimeError::StackUnderflow(self.get_span(), "if".to_string(), 1));
                    }
                }
                Instr::Push(value) => {
                    self.stack.push(*value);
                }
                Instr::Drop => {
                    if let None = self.stack.pop() {
                        return Err(RuntimeError::StackUnderflow(self.get_span(), "drop".to_string(), 1));
                    }
                }
                Instr::Duplicate => {
                    if let Some(value) = self.stack.last() {
                        self.stack.push(*value);
                    } else {
                        return Err(RuntimeError::StackUnderflow(self.get_span(), "dup".to_string(), 1));
                    }
                }
                Instr::Swap => {
                    if let (Some(a), Some(b)) = (self.stack.pop(), self.stack.pop()) {
                        self.stack.push(a);
                        self.stack.push(b);
                    } else {
                        return Err(RuntimeError::StackUnderflow(self.get_span(), "swap".to_string(), 2));
                    }
                }
                Instr::Over => {
                    if let (Some(a), Some(b)) = (self.stack.pop(), self.stack.pop()) {
                        self.stack.push(b);
                        self.stack.push(a);
                        self.stack.push(b);
                    } else {
                        return Err(RuntimeError::StackUnderflow(self.get_span(), "over".to_string(), 2));
                    }
                }
                Instr::Rotate => {
                    if let (Some(a), Some(b), Some(c)) = (self.stack.pop(), self.stack.pop(), self.stack.pop()) {
                        self.stack.push(b);
                        self.stack.push(a);
                        self.stack.push(c);
                    } else {
                        return Err(RuntimeError::StackUnderflow(self.get_span(), "rot".to_string(), 3));
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
                        return Err(RuntimeError::EmptyDefinition(self.get_span(), format!("{}", name)));
                    }
                }
                Instr::SetVariable(name) => {
                    if let Some(scope) = self.namespace.last_mut() {
                        if let Some(value) = self.stack.pop() {
                            scope.insert(name.clone(), value);
                        } else {
                            return Err(RuntimeError::StackUnderflow(self.get_span(), format!("{}", name), 1));
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
                            return Err(RuntimeError::InvalidSymbol(self.get_span(), name.clone()));
                        }
                    }
                }
                Instr::SetSpan(span)  => {
                    // Set the current span for error reporting
                    self.span = *span;
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
                Instr::BeginArray => {
                    self.array_stack.push(self.stack.len());
                }
                Instr::EndArray => {
                    let old_stack = self.array_stack.pop().unwrap();
                    let new_stack = self.stack.len();
                    let mut array: Vec<Value> = Vec::new();
                    let len = new_stack - old_stack;
                    for _ in 0..len {
                        if let Some(value) = self.stack.pop() {
                            array.push(value);
                            continue;
                        }
                        unreachable!()
                    }
                    array.reverse();
                    self.arrays.insert(self.array_id, array);
                    self.stack.push(Value::Array(self.array_id));
                    self.array_id += 1;
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