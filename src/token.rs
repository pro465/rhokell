use core::fmt;
use std::fmt::Display;

use crate::error::{Error, ErrorTy, Loc};

pub struct Scanner<'a> {
    loc: Loc,
    peeked: Option<Result<Token, Error>>,
    rest: &'a str,
}

#[derive(Clone, Debug)]
pub struct Token {
    ty: TokenTy,
    loc: Loc,
}

impl Token {
    pub fn ty(self) -> TokenTy {
        self.ty
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenTy {
    Ident(String),
    Lparen,
    Rparen,
    Equal,
    Semi,
    Eof,
}

impl Display for TokenTy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use TokenTy::*;

        let name = match self {
            Ident(s) => format!("identifier `{}`", s),
            x => match x {
                Equal => "token `=`",
                Lparen => "token `(`",
                Rparen => "token `)`",
                Semi => "token `;`",
                Eof => "EOF",
                _ => unreachable!(),
            }
            .to_string(),
        };
        write!(f, "{}", name)
    }
}

impl<'a> Scanner<'a> {
    pub fn new(s: &'a str) -> Self {
        Self {
            loc: Loc::new(),
            peeked: None,
            rest: s,
        }
    }

    pub fn expect_identifier(&mut self) -> Result<(Loc, String), Error> {
        let res = self.next_token()?;
        if let TokenTy::Ident(x) = res.ty {
            Ok((res.loc, x))
        } else {
            Err(Error {
                loc: res.loc,
                ty: ErrorTy::SyntaxError,
                desc: format!("expected idenifier, found {}", res.ty),
            })
        }
    }

    pub fn expect_token(&mut self, token: TokenTy) -> Result<Token, Error> {
        let res = self.next_token()?;
        if res.ty != token {
            Err(Error {
                loc: res.loc,
                ty: ErrorTy::SyntaxError,
                desc: format!("expected {}, found {}", token, res.ty),
            })
        } else {
            Ok(res)
        }
    }

    pub fn loc(&self) -> Loc {
        self.loc.clone()
    }

    pub fn is_token(&mut self, tok: TokenTy) -> Result<bool, Error> {
        if self.peek()?.ty == tok {
            self.expect_token(tok)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn next_token(&mut self) -> Result<Token, Error> {
        self.peeked
            .take()
            .unwrap_or_else(|| self.next_token_internal())
    }

    pub fn peek(&mut self) -> Result<Token, Error> {
        let r = self.next_token();
        self.peeked = Some(r.clone());
        r
    }

    fn next_token_internal(&mut self) -> Result<Token, Error> {
        self.skip_whitespace();

        if self.rest.is_empty() {
            return Ok(Token {
                loc: self.loc(),
                ty: TokenTy::Eof,
            });
        }
        let mut iter = self.rest.char_indices();
        let (_, c) = iter.next().unwrap();

        if is_break(c) {
            use TokenTy::*;

            let ret = Ok(Token {
                loc: self.loc(),
                ty: match c {
                    ';' => Semi,
                    '=' => Equal,
                    '(' => Lparen,
                    ')' => Rparen,
                    _ => {
                        return Err(Error {
                            loc: self.loc(),
                            ty: ErrorTy::SyntaxError,
                            desc: format!("unrecognized character {}", c),
                        })
                    }
                },
            });
            self.skip(c.len_utf8());
            ret
        } else {
            let mut i = self.rest.len();
            for (j, c) in iter {
                if is_break(c) {
                    i = j;
                    break;
                }
            }
            Ok(Token {
                loc: self.loc(),
                ty: self.ident(i),
            })
        }
    }

    fn ident(&mut self, i: usize) -> TokenTy {
        use TokenTy::*;
        let ident = &self.rest[..i];
        self.skip(i);
        Ident(ident.to_string())
    }

    fn skip_whitespace(&mut self) {
        loop {
            let i = self
                .rest
                .char_indices()
                .find(|(_i, c)| !c.is_whitespace())
                .map(|(i, _c)| i)
                .unwrap_or(self.rest.len());
            self.skip(i);
            if self.rest.chars().next() != Some('#') {
                break;
            }
            let i = self
                .rest
                .char_indices()
                .find(|(_i, c)| *c == '\n')
                .map(|(i, _c)| i + 1)
                .unwrap_or(self.rest.len());
            self.skip(i);
        }
    }
    fn skip(&mut self, len: usize) {
        for c in self.rest[..len].chars() {
            self.loc.col();
            if c == '\n' {
                self.loc.new_line();
            }
        }
        self.rest = &self.rest[len..];
    }
}

fn is_break(c: char) -> bool {
    !c.is_alphanumeric() && c != '_'
}
