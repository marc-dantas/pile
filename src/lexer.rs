use crate::error::*;
use std::iter::{Iterator, Peekable};
use std::str::Chars;

#[derive(Debug, PartialEq)]
pub enum TokenKind {
    Word,
    Int,
    Float,
    String,
}

#[derive(Debug)]
pub struct Token {
    pub value: String,
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug)]
pub struct InputFile<'a> {
    pub name: &'a str,
    pub content: Peekable<Chars<'a>>,
}

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
pub struct FileSpan {
    pub filename: String,
    pub line: usize,
    pub col: usize,
}

// Accepts the character after \ and returns the corresponding escaped character
pub fn escape_char(c: char) -> Option<char> {
    match c {
        'n' => Some('\n'),
        'r' => Some('\r'),
        't' => Some('\t'),
        '"' => Some('"'),
        '0' => Some('\0'),
        // TODO: Add more escape options
        _ => None,
    }
}

impl Span {
    pub fn to_filespan(&self, filename: String) -> FileSpan {
        FileSpan {
            filename,
            line: self.line,
            col: self.col,
        }
    }
}

impl<'a> Token {
    pub fn new(value: String, kind: TokenKind, span: Span) -> Token {
        Self { value, kind, span }
    }

    fn is_int_start(target: &char, next: Option<&char>) -> bool {
        // A number starts with a digit or a '-' followed by a digit
        target.is_ascii_digit()
            || (*target == '-' && next.map_or(false, |c| c.is_ascii_digit()))
    }

    fn is_int(target: &char) -> bool {
        matches!(target, '0'..='9')
    }

    fn is_float_start(target: &char, next: Option<&char>) -> bool {
        // A number starts with a digit or a '-' followed by a digit
        target.is_ascii_digit()
            || (*target == '-' && next.map_or(false, |c| c.is_ascii_digit() || c == &'.'))
    }

    fn is_float(target: &char) -> bool {
        matches!(target, '0'..='9' | '.')
    }

    fn is_word(target: &char) -> bool {
        return target.is_ascii();
    }

    fn is_string(target: &char) -> bool {
        return target == &'"';
    }

    fn is_char(target: &char) -> bool {
        return target == &'\'';
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
    pub input: InputFile<'a>,
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
                            self.span.col = 1;
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
                                FileSpan {
                                    filename: self.input.name.to_string(),
                                    line: self.span.line,
                                    col: self.span.col + 2
                                },
                                Some("check if the string was left open unintentionally."),
                                None,
                            );
                        }
                        match d {
                            '\\' => {
                                if let Some(esc) = self.input.content.next() {
                                    if let Some(c) = escape_char(esc) {
                                        buffer.push(c);
                                    } else {
                                        throw(
                                            "token error",
                                            &format!(
                                                "invalid escape character `{esc}` in string literal."
                                            ),
                                            FileSpan {
                                                filename: self.input.name.to_string(),
                                                line: self.span.line,
                                                col: self.span.col + buffer.len() + 1,
                                            },
                                            None,
                                            None,
                                        );
                                    }
                                }
                            }
                            _ => buffer.push(d)
                        }
                    }
                    self.span.col += buffer.len() + 2; // +2 to consider both quote marks
                    return Some(Token::new(
                        buffer,
                        TokenKind::String,
                        Span {
                            line: self.span.line,
                            col: col,
                        },
                    ));
                }
                _ if Token::is_char(&c) => {
                    if let Some(chr) = self.input.content.next() {
                        let mut chr = chr;
                        if chr == '\\' {
                            if let Some(esc) = self.input.content.next() {
                                if let Some(c) = escape_char(esc) {
                                    return Some(Token::new(
                                        (c as i64).to_string(),
                                        TokenKind::Int,
                                        Span {
                                            line: self.span.line,
                                            col: self.span.col,
                                        },
                                    ));
                                } else if !esc.is_whitespace() {
                                    chr = esc;
                                }
                            }
                        }
                        return Some(Token::new(
                            (chr as i64).to_string(),
                            TokenKind::Int,
                            Span {
                                line: self.span.line,
                                col: self.span.col,
                            },
                        ));
                    }
                }
                _ if Token::is_int_start(&c, self.input.content.peek()) => {
                    let col = self.span.col;
                    let mut buffer = String::from(c);
                    let mut is_float = false;
                    while let Some(d) = self.input.content.peek() {
                        if !Token::is_int(&d) && Token::is_float(&d) {
                            is_float = true;
                        } else if !Token::is_int(&d) {
                            if !Token::is_whitespace(&d) {
                                throw(
                                    "token error",
                                    &format!(
                                        "invalid character `{d}` found in integer/float literal."
                                    ),
                                    FileSpan {
                                        filename: self.input.name.to_string(),
                                        line: self.span.line,
                                        col: self.span.col + buffer.len(),
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
                    if is_float {
                        return Some(Token::new(
                            buffer,
                            TokenKind::Float,
                            Span {
                                line: self.span.line,
                                col: col,
                            },
                        ));
                    } else {
                        return Some(Token::new(
                            buffer,
                            TokenKind::Int,
                            Span {
                                line: self.span.line,
                                col: col,
                            },
                        ));
                    }
                }
                _ if Token::is_float_start(&c, self.input.content.peek()) => {
                    let col = self.span.col;
                    let mut buffer = String::from(c);
                    while let Some(d) = self.input.content.peek() {
                        if !Token::is_float(&d) {
                            if !Token::is_whitespace(&d) {
                                throw(
                                    "token error",
                                    &format!("invalid character `{d}` found in float literal."),
                                    FileSpan {
                                        filename: self.input.name.to_string(),
                                        line: self.span.line,
                                        col: self.span.col + buffer.len(),
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
                        TokenKind::Float,
                        Span {
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
                        Span {
                            line: self.span.line,
                            col: col,
                        },
                    ));
                }
                _ => {
                    throw(
                        "token error",
                        &format!("illegal character `{c}` found in file."),
                        FileSpan {
                            filename: self.input.name.to_string(),
                            line: self.span.line,
                            col: self.span.col,
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
