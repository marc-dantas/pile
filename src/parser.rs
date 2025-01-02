use crate::lexer::{Lexer, Token, TokenKind, TokenSpan};

pub fn is_op(value: &str) -> bool {
    matches!(
        value,
        "dup"
            | "drop"
            | "swap"
            | "over"
            | "rot"
            | "dump"
            | "+"
            | "-"
            | "*"
            | "/"
            | ">"
            | "<"
            | "="
            | ">="
            | "<="
            | "!="
            | ">>"
            | "<<"
            | "|"
            | "&"
            | "~"
            | "**"
    )
}

pub fn is_reserved_word(value: &str) -> bool {
    matches!(
        value,
        "if" | "loop" | "proc" | "end" | "else" | "def" | "stop"
    )
}

// don't know if this really works in all possibilities, i have to test it
pub fn is_valid_identifier(value: &str) -> bool {
    !value.chars().next().map_or(false, |c| c.is_digit(10))
        && value.chars().all(|c| c.is_alphanumeric() || c == '_')
        && !is_reserved_word(value)
        && !is_op(value)
}

#[derive(Debug)]
pub enum OpKind {
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
    BNot,
    Swap,
    Over,
    Dump,
    Dup,
    Rot,
    Drop,
    Stop,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum Node {
    Number(f64, TokenSpan),
    String(String, TokenSpan),
    Proc(String, Vec<Node>, TokenSpan),
    Def(String, Vec<Node>, TokenSpan),
    If(Vec<Node>, Option<Vec<Node>>, TokenSpan),
    Loop(Vec<Node>, TokenSpan),
    Operation(OpKind, TokenSpan),
    Word(String, TokenSpan),
}

pub type ProgramTree = Vec<Node>;

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_span: Option<TokenSpan>,
}

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken(TokenSpan, String, String),
    UnexpectedEOF(TokenSpan, String),
    UnterminatedBlock(TokenSpan, String),
    UnmatchedBlock(TokenSpan),
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        Self {
            lexer: lexer,
            current_span: None,
        }
    }

    pub fn parse(&mut self) -> Result<ProgramTree, ParseError> {
        let mut exprs = Vec::new();
        while let Some(token) = self.lexer.next() {
            self.current_span = Some(token.span.clone());
            exprs.push(self.parse_expr(token)?);
        }
        Ok(exprs)
    }

    fn parse_expr(&mut self, token: Token) -> Result<Node, ParseError> {
        match token.kind {
            TokenKind::Number => Ok(Node::Number(token.value.parse().unwrap(), token.span)),
            TokenKind::Word => match token.value.as_str() {
                "proc" => self.parse_proc(),
                "def" => self.parse_def(),
                "if" => self.parse_if(),
                "loop" => self.parse_loop(),
                "end" => Err(ParseError::UnmatchedBlock(
                    self.current_span
                        .clone()
                        .unwrap_or_else(|| token.span.clone()),
                )),
                "+" => Ok(Node::Operation(OpKind::Add, token.span)),
                "-" => Ok(Node::Operation(OpKind::Sub, token.span)),
                "*" => Ok(Node::Operation(OpKind::Mul, token.span)),
                "/" => Ok(Node::Operation(OpKind::Div, token.span)),
                "%" => Ok(Node::Operation(OpKind::Mod, token.span)),
                "**" => Ok(Node::Operation(OpKind::Exp, token.span)),
                "=" => Ok(Node::Operation(OpKind::Eq, token.span)),
                "!=" => Ok(Node::Operation(OpKind::Ne, token.span)),
                ">" => Ok(Node::Operation(OpKind::Gt, token.span)),
                "<" => Ok(Node::Operation(OpKind::Lt, token.span)),
                "<=" => Ok(Node::Operation(OpKind::Le, token.span)),
                ">=" => Ok(Node::Operation(OpKind::Ge, token.span)),
                "|" => Ok(Node::Operation(OpKind::Bor, token.span)),
                "&" => Ok(Node::Operation(OpKind::Band, token.span)),
                ">>" => Ok(Node::Operation(OpKind::Shr, token.span)),
                "<<" => Ok(Node::Operation(OpKind::Shl, token.span)),
                "~" => Ok(Node::Operation(OpKind::BNot, token.span)),
                "dup" => Ok(Node::Operation(OpKind::Dup, token.span)),
                "drop" => Ok(Node::Operation(OpKind::Drop, token.span)),
                "swap" => Ok(Node::Operation(OpKind::Swap, token.span)),
                "over" => Ok(Node::Operation(OpKind::Over, token.span)),
                "rot" => Ok(Node::Operation(OpKind::Rot, token.span)),
                "dump" => Ok(Node::Operation(OpKind::Dump, token.span)),
                "stop" => Ok(Node::Operation(OpKind::Stop, token.span)),
                x if is_valid_identifier(x) => Ok(Node::Word(token.value, token.span)),
                _ => Err(ParseError::UnexpectedToken(
                    token.span.clone(),
                    token.value,
                    "number, word, string, or operation".to_string(),
                )),
            },
            TokenKind::String => Ok(Node::String(token.value, token.span)),
        }
    }

    fn parse_proc(&mut self) -> Result<Node, ParseError> {
        let proc_name = self.lexer.next().ok_or_else(|| {
            let span = self.current_span.clone().unwrap();
            ParseError::UnexpectedEOF(span, "valid identifier".to_string())
        })?;

        if !is_valid_identifier(&proc_name.value) {
            return Err(ParseError::UnexpectedToken(
                proc_name.span.clone(),
                proc_name.value,
                "valid identifier".to_string(),
            ));
        }

        let mut body = Vec::new();

        while let Some(token) = self.lexer.next() {
            if token.value == "end" {
                return Ok(Node::Proc(proc_name.value, body, token.span));
            }
            body.push(self.parse_expr(token)?);
        }

        Err(ParseError::UnterminatedBlock(
            proc_name.span.clone(),
            "proc".to_string(),
        ))
    }

    fn parse_def(&mut self) -> Result<Node, ParseError> {
        let def_name = self.lexer.next().ok_or_else(|| {
            let span = self.current_span.clone().unwrap_or_else(|| TokenSpan {
                filename: "unknown".to_string(),
                line: 0,
                col: 0,
            });
            ParseError::UnexpectedEOF(span, "valid identifier".to_string())
        })?;

        if !is_valid_identifier(&def_name.value) {
            return Err(ParseError::UnexpectedToken(
                def_name.span.clone(),
                def_name.value,
                "valid identifier".to_string(),
            ));
        }

        let mut body = Vec::new();

        while let Some(token) = self.lexer.next() {
            if token.value == "end" {
                return Ok(Node::Def(def_name.value, body, token.span));
            }
            body.push(self.parse_expr(token)?);
        }

        Err(ParseError::UnterminatedBlock(
            def_name.span.clone(),
            "proc".to_string(),
        ))
    }

    fn parse_if(&mut self) -> Result<Node, ParseError> {
        let mut if_body = Vec::new();
        let else_body = None;

        while let Some(token) = self.lexer.next() {
            if token.value == "else" {
                let mut else_block = Vec::new();
                while let Some(token) = self.lexer.next() {
                    if token.value == "end" {
                        return Ok(Node::If(if_body, Some(else_block), token.span));
                    }
                    else_block.push(self.parse_expr(token)?);
                }
                return Err(ParseError::UnterminatedBlock(
                    token.span.clone(),
                    "else".to_string(),
                ));
            } else if token.value == "end" {
                return Ok(Node::If(if_body, else_body, token.span));
            }
            if_body.push(self.parse_expr(token)?);
        }

        Err(ParseError::UnterminatedBlock(
            self.current_span.clone().unwrap(),
            "if".to_string(),
        ))
    }

    fn parse_loop(&mut self) -> Result<Node, ParseError> {
        let mut body = Vec::new();

        while let Some(token) = self.lexer.next() {
            if token.value == "end" {
                return Ok(Node::Loop(body, token.span));
            }
            body.push(self.parse_expr(token)?);
        }

        Err(ParseError::UnterminatedBlock(
            self.current_span.clone().unwrap(),
            "loop".to_string(),
        ))
    }
}
