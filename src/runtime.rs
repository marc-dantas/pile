use crate::{
    lexer::TokenSpan,
    parser::{Node, OpKind, ProgramTree},
};
use std::collections::VecDeque;

#[derive(Debug)]
pub enum Data {
    String(String),
    Number(f64),
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

pub fn binaryop_readable(x: BinaryOp) -> String {
    match x {
        BinaryOp::Add => String::from("+"),
        BinaryOp::Sub => String::from("-"),
        BinaryOp::Mul => String::from("*"),
        BinaryOp::Div => String::from("/"),
        BinaryOp::Gt => String::from(">"),
        BinaryOp::Lt => String::from("<"),
        BinaryOp::Eq => String::from("="),
        BinaryOp::Ge => String::from(">="),
        BinaryOp::Le => String::from("<="),
        BinaryOp::Ne => String::from("!="),
        BinaryOp::Shl => String::from(">>"),
        BinaryOp::Shr => String::from("<<"),
        BinaryOp::Bor => String::from("|"),
        BinaryOp::Band => String::from("&"),
        BinaryOp::Swap => String::from("swap"),
        BinaryOp::Over => String::from("over"),
    }
}

pub enum UnaryOp {
    Dump,
    Dup,
    Drop,
}

pub fn unaryop_readable(x: UnaryOp) -> String {
    match x {
        UnaryOp::Dump => String::from("dump"),
        UnaryOp::Dup => String::from("dup"),
        UnaryOp::Drop => String::from("drop"),
    }
}

pub type Stack = VecDeque<Data>;

#[derive(Debug)]
pub enum RuntimeError {
    ProcedureError {
        call: TokenSpan, // TokenSpan where the procedure was called
        inner: Box<RuntimeError>, // the original error inside the procedure
    },
    StackUnderflow(TokenSpan, String, usize), // when there's too few data on the stack to perform operation
    UnexpectedType(TokenSpan, String, String, String), // when there's an operation tries to operate with an unsupported or an invalid datatype
    InvalidWord(TokenSpan, String), // used when a word doesn't correspond a valid identifier
    ProcRedefinition(TokenSpan, String), // used when a procedure name is already taken
    DefRedefinition(TokenSpan, String), // used when a definition name is already taken
    EmptyDefinition(TokenSpan, String), // used when a definition has empty body
}

pub struct Runtime<'a> {
    input: &'a ProgramTree,
    stack: Stack,
    namespace: Namespace<'a>,
    stop: bool
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
            stop: false
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
    
    fn unop(&mut self, span: TokenSpan, x: UnaryOp) -> Result<(), RuntimeError> {
        if let Some(a) = self.pop() {
            match a {
                Data::Number(n) => match x {
                    UnaryOp::Dump => println!("{}", n),
                    UnaryOp::Dup => {
                        self.push_number(n);
                        self.push_number(n);
                    }
                    UnaryOp::Drop => {}
                },
                Data::String(s) => match x {
                    UnaryOp::Dump => println!("{}", s),
                    UnaryOp::Dup => {
                        self.push_string(s.clone());
                        self.push_string(s);
                    }
                    UnaryOp::Drop => {}
                },
            }
        } else {
            return Err(RuntimeError::StackUnderflow(span, unaryop_readable(x), 1));
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
                (Data::String(s1), Data::String(s2)) => match x {
                    BinaryOp::Add => self.push_string(s1 + &s2),
                    BinaryOp::Eq => self.push_number((s1 == s2) as i32 as f64),
                    BinaryOp::Ne => self.push_number((s1 != s2) as i32 as f64),
                    // other comparison operators are not (should not be) supported for strings
                    // BinaryOp::Lt => self.push_number((s1 < s2) as i32 as f64),
                    // BinaryOp::Gt => self.push_number((s1 > s2) as i32 as f64),
                    // BinaryOp::Le => self.push_number((s1 <= s2) as i32 as f64),
                    // BinaryOp::Ge => self.push_number((s1 >= s2) as i32 as f64),
                    BinaryOp::Swap => {
                        self.push_string(s1);
                        self.push_string(s2);
                    }
                    BinaryOp::Over => {
                        self.push_string(s2.clone());
                        self.push_string(s1.clone());
                        self.push_string(s2);
                    }
                    _ => {
                        return Err(RuntimeError::UnexpectedType(
                            span,
                            binaryop_readable(x),
                            "(number, number)".to_string(),
                            "(string, string)".to_string(),
                        ))
                    }
                },
                (Data::String(_), Data::Number(_)) => {
                    return Err(RuntimeError::UnexpectedType(
                        span,
                        binaryop_readable(x),
                        "(number, number) or (string, string)".to_string(),
                        "(number, string)".to_string(),
                    ));
                }
                (Data::Number(_), Data::String(_)) => {
                    return Err(RuntimeError::UnexpectedType(
                        span,
                        binaryop_readable(x),
                        "(number, number) or (string, string)".to_string(),
                        "(string, number)".to_string(),
                    ));
                }
            }
        } else {
            return Err(RuntimeError::StackUnderflow(span, binaryop_readable(x), 2));
        }
        Ok(())
    }

    fn run_node(&mut self, n: &'a Node) -> Result<(), RuntimeError> {
        match n {
            Node::If(i, e, s) => {
                if let Some(a) = self.pop() {
                    match a {
                        Data::Number(n) => {
                            if n != 0.0 {
                                self.run_block(i)?;
                            } else {
                                if let Some(els) = e {
                                    self.run_block(els)?;
                                }
                            }
                        }
                        // TODO: Add bool type and maybe truthy values
                        Data::String(_) => {
                            return Err(RuntimeError::UnexpectedType(
                                s.clone(),
                                "if".to_string(),
                                "(number)".to_string(),
                                "(string)".to_string(),
                            ));
                        }
                    }
                } else {
                    return Err(RuntimeError::StackUnderflow(
                        s.clone(),
                        "if".to_string(),
                        1
                    ));
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
                    OpKind::Dup => self.unop(s, UnaryOp::Dup)?,
                    OpKind::Drop => self.unop(s, UnaryOp::Drop)?,
                    OpKind::Dump => self.unop(s, UnaryOp::Dump)?,
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
                    OpKind::Stop => { self.stop = true; }
                }
            }
            Node::Word(w, s) => {
                if let Some(p) = self.namespace.procs.iter().find(|p| p.0 == *w) {
                    if let Err(e) = self.run_block(&p.1) {
                        return Err(RuntimeError::ProcedureError { call: s.clone(), inner: Box::new(e) })
                    }
                } else if let Some(d) = self.namespace.defs.iter().find(|p| p.0 == *w) {
                    match &d.1 {
                        Data::Number(n) => self.push_number(*n),
                        Data::String(s) => self.push_string(String::from(s)),
                    }
                } else {
                    return Err(RuntimeError::InvalidWord(s.clone(), w.to_string()));
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
