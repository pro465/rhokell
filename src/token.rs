use crate::{
    alloc::{Alloc, DisplayWithAlloc, Id},
    error::{Error, ErrorTy, Loc},
};

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
    Ident(Id),
    Lparen,
    Rparen,
    Equal,
    Semi,
    Eof,
}

impl DisplayWithAlloc for TokenTy {
    fn display(&self, alloc: &Alloc, s: &mut String) {
        use TokenTy::*;

        let name = match self {
            Ident(id) => {
                s.push_str("identifier `");
                s.push_str(alloc.get_string(id));
                s.push('`');
                return;
            }
            x => match x {
                Equal => "token `=`",
                Lparen => "token `(`",
                Rparen => "token `)`",
                Semi => "token `;`",
                Eof => "EOF",
                _ => unreachable!(),
            },
        };
        s.push_str(name)
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

    pub fn expect_identifier(&mut self, alloc: &mut Alloc) -> Result<(Loc, Id), Error> {
        let res = self.next_token(alloc)?;
        if let TokenTy::Ident(x) = res.ty {
            Ok((res.loc, x))
        } else {
            Err(Error {
                loc: res.loc,
                ty: ErrorTy::SyntaxError,
                desc: format!("expected identifier, found {}", res.ty.to_string(alloc)),
            })
        }
    }

    pub fn expect_token(&mut self, alloc: &mut Alloc, token: TokenTy) -> Result<Token, Error> {
        let res = self.next_token(alloc)?;
        if res.ty != token {
            Err(Error {
                loc: res.loc,
                ty: ErrorTy::SyntaxError,
                desc: format!(
                    "expected {}, found {}",
                    token.to_string(alloc),
                    res.ty.to_string(alloc)
                ),
            })
        } else {
            Ok(res)
        }
    }

    pub fn loc(&self) -> Loc {
        self.loc.clone()
    }

    pub fn is_token(&mut self, alloc: &mut Alloc, tok: TokenTy) -> Result<bool, Error> {
        if self.peek(alloc)?.ty == tok {
            self.expect_token(alloc, tok)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn next_token(&mut self, alloc: &mut Alloc) -> Result<Token, Error> {
        self.peeked
            .take()
            .unwrap_or_else(|| self.next_token_internal(alloc))
    }

    pub fn peek(&mut self, alloc: &mut Alloc) -> Result<Token, Error> {
        let r = self.next_token(alloc);
        self.peeked = Some(r.clone());
        r
    }

    fn next_token_internal(&mut self, alloc: &mut Alloc) -> Result<Token, Error> {
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
                ty: self.ident(i, alloc),
            })
        }
    }

    fn ident(&mut self, i: usize, alloc: &mut Alloc) -> TokenTy {
        use TokenTy::*;
        let id = alloc.alloc_or_get(&self.rest[..i]);
        self.skip(i);
        Ident(id)
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
    "()=;".contains(c) || c.is_whitespace()
}
