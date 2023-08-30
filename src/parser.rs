use std::{fmt::Display, rc::Rc};

use crate::{
    error::{Error, Loc},
    token::{Scanner, TokenTy},
};

#[derive(Clone, Debug)]
pub struct Def {
    pub name: String,
    pub loc: Loc,
    pub(crate) pat: Expr,
    pub(crate) rep: Expr,
}

#[derive(Clone, Debug)]
pub enum Expr {
    Var { name: String, loc: Loc },
    // reduced function
    RedFunc(Rc<Func>),
    // unreduced function
    Func(Func),
}

#[derive(Clone, Debug, Default)]
pub struct Func {
    pub(crate) loc: Loc,
    pub(crate) name: String,
    pub(crate) args: Vec<Expr>,
}

impl Expr {
    pub(crate) fn loc(&self) -> Loc {
        match self {
            Expr::Var { loc, .. } => *loc,
            Expr::Func(Func { loc, .. }) => *loc,
            Expr::RedFunc(f) => f.loc,
        }
    }
}

impl PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Expr::Var { name, .. }, Expr::Var { name: name2, .. }) => name == name2,
            (Expr::RedFunc(f1), Expr::RedFunc(f2)) => f1 == f2,
            (Expr::Func(f1), Expr::Func(f2)) => f1 == f2,
            _ => false,
        }
    }
}

impl PartialEq for Func {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.args == other.args
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        stacker::maybe_grow(32 * 1024, 1024 * 1024, || match self {
            Expr::RedFunc(fun) => {
                let Func { name, args, .. } = &**fun;
                write!(f, "{}", name)?;
                if args.len() != 1 {
                    write!(f, "(")?;
                } else {
                    write!(f, " ")?;
                }
                for (i, arg) in args.iter().enumerate() {
                    write!(f, "{}", &arg)?;
                    if i < args.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                if args.len() != 1 {
                    write!(f, ")")?;
                }
                Ok(())
            }
            Expr::Func(Func { name, args, .. }) => {
                write!(f, "{}", name)?;
                if args.len() != 1 {
                    write!(f, "(")?;
                } else {
                    write!(f, " ")?;
                }
                for (i, arg) in args.iter().enumerate() {
                    write!(f, "{}", &arg)?;
                    if i < args.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                if args.len() != 1 {
                    write!(f, ")")?;
                }
                Ok(())
            }
            Expr::Var { name, .. } => write!(f, "{}", name),
        })
    }
}

pub struct Parser<'a> {
    pub(crate) sc: Scanner<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(sc: Scanner<'a>) -> Self {
        Self { sc }
    }
    pub fn parse_def(&mut self) -> Result<Option<Def>, Error> {
        if self.sc.peek()?.ty() == TokenTy::Eof {
            return Ok(None);
        }
        let pat = self.parse_expr::<true>()?;
        let (name, loc) = match &pat {
            Expr::Func(f) => (f.name.clone(), f.loc),
            _ => unreachable!(),
        };
        self.sc.expect_token(TokenTy::Equal)?;
        let rep = self.parse_expr::<false>()?;
        self.sc.expect_token(TokenTy::Semi)?;

        Ok(Some(Def {
            name,
            loc,
            pat,
            rep,
        }))
    }

    pub fn parse_expr<const B: bool>(&mut self) -> Result<Expr, Error> {
        let (loc, name) = self.sc.expect_identifier()?;

        if self.sc.is_identifier()? {
            let args = vec![self.parse_expr::<false>()?];
            return Ok(Expr::Func(Func { name, args, loc }));
        }

        if B {
            self.sc.expect_token(TokenTy::Lparen)?;
        } else if !self.sc.is_token(TokenTy::Lparen)? {
            return Ok(Expr::Var { name, loc });
        }
        let mut args = Vec::new();
        if !self.sc.is_token(TokenTy::Rparen)? {
            loop {
                let s = self.parse_expr::<false>()?;
                args.push(s);
                if self.expect_commma_or(TokenTy::Rparen)? {
                    break;
                }
            }
        }

        Ok(Expr::Func(Func { name, args, loc }))
    }

    fn expect_commma_or(&mut self, b: TokenTy) -> Result<bool, Error> {
        let mut res = self.sc.expect_one(&[TokenTy::Comma, b.clone()])?.ty();
        if res == TokenTy::Comma {
            res = self.sc.peek()?.ty();
            if res == b {
                self.sc.expect_token(b.clone())?;
            }
        }

        Ok(res == b)
    }
}
