use crate::error::*;
use std::iter::{Iterator, Peekable};
use std::str::Chars;

#[derive(Debug, PartialEq)]
pub enum TokenKind {
    Word,
    Number,
    String,
}

#[derive(Debug)]
pub struct Token {
    pub value: String,
    pub kind: TokenKind,
    pub span: TokenSpan,
}

#[derive(Debug)]
pub struct InputFile<'a> {
    pub name: &'a str,
    pub content: Peekable<Chars<'a>>,
}

#[derive(Debug)]
pub struct Span {
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
pub struct TokenSpan {
    pub filename: String,
    pub line: usize,
    pub col: usize,
}

impl<'a> Token {
    pub fn new(value: String, kind: TokenKind, span: TokenSpan) -> Token {
        Self { value, kind, span }
    }

    fn is_number_start(target: &char, next: Option<&char>) -> bool {
        // A number starts with a digit or a '-' followed by a digit
        matches!(target, '0'..='9' | '.') ||
        (*target == '-' && next.map_or(false, |c| c == &'.' || c.is_ascii_digit()))
    }

    fn is_number(target: &char) -> bool {
        matches!(target, '0'..='9' | '.')
    }

    fn is_word(target: &char) -> bool {
        return target.is_ascii();
    }

    fn is_string(target: &char) -> bool {
        return target == &'"';
    }

    fn is_newline(target: &char) -> bool {
        return target == &'\n';
    }

    fn is_whitespace(target: &char) -> bool {
        target.is_whitespace()
    }

    fn is_comment(target: &char) -> bool {
        target == &'#'
    }
}

pub struct Lexer<'a> {
    input: InputFile<'a>,
    span: Span,
}

impl<'a> Lexer<'a> {
    pub fn new(input: InputFile<'a>, span: Span) -> Self {
        Self { input, span }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(c) = self.input.content.next() {
            match c {
                _ if Token::is_newline(&c) => {
                    self.span.line += 1;
                    self.span.col = 1;
                    continue;
                }
                _ if Token::is_whitespace(&c) => {
                    self.span.col += 1;
                    continue;
                }
                _ if Token::is_comment(&c) => {
                    while let Some(d) = self.input.content.next() {
                        if Token::is_newline(&d) {
                            self.span.line += 1;
                            break;
                        }
                    }
                }
                _ if Token::is_string(&c) => {
                    let col: usize = self.span.col;
                    let mut buffer = String::new();
                    while let Some(d) = self.input.content.next() {
                        if Token::is_string(&d) {
                            break;
                        } else if self.input.content.peek().is_none() {
                            throw(
                                "token error",
                                &format!(
                                    "expected closing quotation mark (\") for string literal \"{}\".",
                                    buffer.clone() + &String::from(d)
                                ),
                                TokenSpan {
                                    filename: self.input.name.to_string(),
                                    line: self.span.line,
                                    col: self.span.col + 2    
                                },
                                Some("check if the string was left open unintentionally."),
                                None,
                            );
                        }
                        buffer.push(d);
                    }
                    self.span.col += buffer.len() + 2; // +2 to consider both quote marks
                    return Some(Token::new(
                        buffer,
                        TokenKind::String,
                        TokenSpan {
                            filename: self.input.name.to_string(),
                            line: self.span.line,
                            col: col,
                        },
                    ));
                }
                _ if Token::is_number_start(&c, self.input.content.peek()) => {
                    let col = self.span.col;
                    let mut buffer = String::from(c);
                    while let Some(d) = self.input.content.peek() {
                        if !Token::is_number(&d) {
                            if !Token::is_whitespace(&d) {
                                throw(
                                    "token error",
                                    &format!("invalid character `{d}` found in number literal."),
                                    TokenSpan {
                                        filename: self.input.name.to_string(),
                                        line: self.span.line,
                                        col: self.span.col + buffer.len()    
                                    },
                                    None,
                                    None,
                                );
                            }
                            break;
                        }
                        buffer.push(*d);
                        self.input.content.next();
                    }
                    self.span.col += buffer.len();
                    return Some(Token::new(
                        buffer,
                        TokenKind::Number,
                        TokenSpan {
                            filename: self.input.name.to_string(),
                            line: self.span.line,
                            col: col,
                        },
                    ));
                }
                _ if Token::is_word(&c) => {
                    let col: usize = self.span.col;
                    let mut buffer = String::from(c);
                    while let Some(d) = self.input.content.peek() {
                        if Token::is_whitespace(&d) {
                            break;
                        }
                        buffer.push(*d);
                        self.input.content.next();
                    }
                    self.span.col += buffer.len();
                    return Some(Token::new(
                        buffer,
                        TokenKind::Word,
                        TokenSpan {
                            filename: self.input.name.to_string(),
                            line: self.span.line,
                            col: col,
                        },
                    ));
                }
                _ => {
                    throw(
                        "token error",
                        &format!("illegal character `{c}` found in file."),
                        TokenSpan {
                            filename: self.input.name.to_string(),
                            line: self.span.line,
                            col: self.span.col    
                        },
                        None,
                        None,
                    );
                }
            }
        }
        None
    }
}
