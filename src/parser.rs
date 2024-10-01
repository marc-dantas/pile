use crate::lexer::{Lexer, Token, TokenKind, TokenSpan};

fn is_reserved_word(value: &str) -> bool {
    matches!(value, "if" | "loop" | "proc" | "end" | "else")
}

// don't know if this really works in all possibilities, i have to test it
fn is_valid_identifier(value: &str) -> bool {
    !value.chars().next().map_or(false, |c| c.is_digit(10))
        && value.chars().all(|c| c.is_alphanumeric() || c == '_')
        && !is_reserved_word(value)
}

#[derive(Debug)]
pub enum Node {
    Number(f64, TokenSpan),
    String(String, TokenSpan),
    Procedure(String, Vec<Node>, TokenSpan),
    If(Vec<Node>, Option<Vec<Node>>, TokenSpan),
    Loop(Vec<Node>, TokenSpan),
    Operation(String, TokenSpan),
    // Word(String, TokenSpan)
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
                "if" => self.parse_if(),
                "loop" => self.parse_loop(),
                "end" => Err(ParseError::UnmatchedBlock(
                    self.current_span
                        .clone()
                        .unwrap_or_else(|| token.span.clone()),
                )),
                // value if is_valid_identifier(value) => Ok(Node::Word(token.value, token.span)),
                
                // expand this match to more Expr variants for each operation (or maybe implement an OpKind-like thing)
                _ => Ok(Node::Operation(token.value, token.span))
            },
            TokenKind::String => Ok(Node::String(token.value, token.span)),
        }
    }

    fn parse_proc(&mut self) -> Result<Node, ParseError> {
        let proc_name = self.lexer.next().ok_or_else(|| {
            let span = self.current_span.clone().unwrap_or_else(|| TokenSpan {
                filename: "unknown".to_string(),
                line: 0,
                col: 0,
            });
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
            self.current_span = Some(token.span.clone());
            if token.value == "end" {
                return Ok(Node::Procedure(proc_name.value, body, token.span));
            }
            body.push(self.parse_expr(token)?);
        }

        Err(ParseError::UnterminatedBlock(
            self.current_span
                .clone()
                .unwrap_or_else(|| proc_name.span.clone()),
            "proc".to_string(),
        ))
    }

    fn parse_if(&mut self) -> Result<Node, ParseError> {
        let mut if_body = Vec::new();
        let else_body = None;

        while let Some(token) = self.lexer.next() {
            self.current_span = Some(token.span.clone());
            if token.value == "else" {
                let mut else_block = Vec::new();
                while let Some(token) = self.lexer.next() {
                    self.current_span = Some(token.span.clone());
                    if token.value == "end" {
                        return Ok(Node::If(if_body, Some(else_block), token.span));
                    }
                    else_block.push(self.parse_expr(token)?);
                }
                return Err(ParseError::UnterminatedBlock(
                    self.current_span
                        .clone()
                        .unwrap_or_else(|| token.span.clone()),
                    "else".to_string(),
                ));
            } else if token.value == "end" {
                return Ok(Node::If(if_body, else_body, token.span));
            }
            if_body.push(self.parse_expr(token)?);
        }

        Err(ParseError::UnterminatedBlock(
            self.current_span.clone().unwrap_or_else(|| TokenSpan {
                filename: "unknown".to_string(),
                line: 0,
                col: 0,
            }),
            "if".to_string(),
        ))
    }

    fn parse_loop(&mut self) -> Result<Node, ParseError> {
        let mut body = Vec::new();

        while let Some(token) = self.lexer.next() {
            self.current_span = Some(token.span.clone());
            if token.value == "end" {
                return Ok(Node::Loop(body, token.span));
            }
            body.push(self.parse_expr(token)?);
        }

        Err(ParseError::UnterminatedBlock(
            self.current_span.clone().unwrap_or_else(|| TokenSpan {
                filename: "unknown".to_string(),
                line: 0,
                col: 0,
            }),
            "loop".to_string(),
        ))
    }
}
