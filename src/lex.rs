use std::{fmt::Display, iter::Peekable, str::Chars};

#[derive(Debug, Clone, Copy)]
pub struct Loc {
    pub col: usize,
    pub row: usize,
}

impl Loc {
    pub fn new() -> Loc {
        Loc { col: 1, row: 1 }
    }
}

impl Display for Loc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.row, self.col)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    LitBool,
    LitInt,
    Let,
    In,
    Var,
    BackSlash,
    Eq,
    OpenP,
    ClosP,
    Arrow,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub loc: Loc,
    pub kind: TokenKind,
    pub value: String,
}

pub struct Lexer<'c> {
    chars: Peekable<Chars<'c>>,
    loc: Loc,
}

impl<'c> Iterator for Lexer<'c> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.lex()
    }
}

impl<'c> Lexer<'c> {
    pub fn new(source: &'c str) -> Self {
        Self {
            chars: source.chars().peekable(),
            loc: Loc::new(),
        }
    }

    pub fn lex(&mut self) -> Option<Token> {
        let ch = *self.chars.peek()?;

        if ch.is_whitespace() {
            self.skip_whitespace();
            self.lex()
        } else if ch.is_alphabetic() {
            self.lex_var_or_keyword()
        } else if ch.is_numeric() {
            self.lex_int()
        } else if ch == '\\' {
            self.loc.col += 1;
            self.chars.next();
            Some(Token {
                loc: self.loc,
                kind: TokenKind::BackSlash,
                value: "\\".to_string(),
            })
        } else if ch == '=' {
            self.loc.col += 1;
            self.chars.next();
            Some(Token {
                loc: self.loc,
                kind: TokenKind::Eq,
                value: "=".to_string(),
            })
        } else if ch == '(' {
            self.loc.col += 1;
            self.chars.next();
            Some(Token {
                loc: self.loc,
                kind: TokenKind::OpenP,
                value: "(".to_string(),
            })
        } else if ch == ')' {
            self.loc.col += 1;
            self.chars.next();
            Some(Token {
                loc: self.loc,
                kind: TokenKind::ClosP,
                value: ")".to_string(),
            })
        } else if ch == '-' {
            self.chars.next();
            let ra = self.chars.next()?;
            if ra == '>' {
                self.loc.col += 2;
                return Some(Token {
                    loc: self.loc,
                    kind: TokenKind::Arrow,
                    value: "->".to_string(),
                });
            } else {
                return None;
            }
        } else {
            return None;
        }
    }

    pub fn skip_whitespace(&mut self) {
        while let Some(ch) = self.chars.peek() {
            if ch.is_whitespace() {
                if *ch == '\n' {
                    self.loc.col = 1;
                    self.loc.row += 1;
                } else {
                    self.loc.col += 1;
                }
                self.chars.next();
            } else {
                break;
            }
        }
    }

    pub fn lex_var_or_keyword(&mut self) -> Option<Token> {
        let token_loc = self.loc;
        let mut value = String::new();
        while let Some(&ch) = self.chars.peek() {
            if ch.is_alphanumeric() {
                self.loc.col += 1;
                value.push(ch);
                self.chars.next();
            } else {
                break;
            }
        }

        let kind = match value.as_str() {
            "true" | "false" => TokenKind::LitBool,
            "let" => TokenKind::Let,
            "in" => TokenKind::In,
            _ => TokenKind::Var,
        };

        Some(Token {
            loc: token_loc,
            kind,
            value,
        })
    }

    pub fn lex_int(&mut self) -> Option<Token> {
        let token_loc = self.loc;
        let mut value = String::new();
        while let Some(&ch) = self.chars.peek() {
            if ch.is_numeric() {
                self.loc.col += 1;
                value.push(ch);
                self.chars.next();
            } else {
                break;
            }
        }

        Some(Token {
            loc: token_loc,
            kind: TokenKind::LitInt,
            value,
        })
    }
}
