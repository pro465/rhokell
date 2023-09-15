use std::{fmt::Display, rc::Rc};

use crate::{
    error::{Error, ErrorTy, Loc},
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
    Fun { name: String, loc: Loc },
    Var { name: String, loc: Loc },
    // reduced function
    RedApp(Rc<App>),
    // unreduced function
    App(Box<App>),
}

#[derive(Clone, Debug, Default)]
pub struct App {
    pub(crate) name: Rc<String>,
    pub(crate) loc: Loc,
    pub(crate) f: Expr,
    pub(crate) arg: Expr,
}

impl Expr {
    pub(crate) fn loc(&self) -> Loc {
        match self {
            Expr::Var { loc, .. } | Expr::Fun { loc, .. } => *loc,
            Expr::App(f) => f.loc,
            Expr::RedApp(f) => f.loc,
        }
    }
    fn display(&self, parens: bool, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        crate::with_stacker(|| {
            let parens = parens && !matches!(self, Expr::Var { .. });
            if parens {
                write!(fmt, "(")?;
            }

            match self {
                Expr::RedApp(fun) => {
                    let App { f, arg, .. } = &**fun;
                    f.display(false, fmt)?;
                    write!(fmt, " ")?;
                    arg.display(true, fmt)
                }
                Expr::App(fun) => {
                    let App { f, arg, .. } = &**fun;
                    f.display(false, fmt)?;
                    write!(fmt, " ")?;
                    arg.display(true, fmt)
                }
                Expr::Var { name, .. } => write!(fmt, "{}", name),
                Expr::Fun { name, .. } => write!(fmt, "{}", name),
            }?;
            if parens {
                write!(fmt, ")")?;
            }

            Ok(())
        })
    }
}

impl Default for Expr {
    fn default() -> Self {
        Expr::Var {
            name: String::default(),
            loc: Loc::default(),
        }
    }
}

impl PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Expr::Var { name, .. }, Expr::Var { name: name2, .. }) => name == name2,
            (Expr::Fun { name, .. }, Expr::Fun { name: name2, .. }) => name == name2,
            (Expr::RedApp(f1), Expr::RedApp(f2)) => f1 == f2,
            (Expr::App(f1), Expr::App(f2)) => f1 == f2,
            _ => false,
        }
    }
}

impl PartialEq for App {
    fn eq(&self, other: &Self) -> bool {
        self.f == other.f && self.arg == other.arg
    }
}

impl Display for Expr {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.display(true, fmt)
    }
}

impl Drop for App {
    fn drop(&mut self) {
        use std::mem::take;
        crate::with_stacker(|| {
            take(&mut self.f);
            take(&mut self.arg);
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
        let (name, loc, pat) = self.parse_expr(false)?;
        if let Expr::Var { .. } = pat {
            return Err(Error {
                loc,
                ty: ErrorTy::SyntaxError,
                desc: "bare variables are not allowed".into(),
            });
        }
        let name = (&*name).clone();
        self.sc.expect_token(TokenTy::Equal)?;
        let (_, _, rep) = self.parse_expr(false)?;
        self.sc.expect_token(TokenTy::Semi)?;

        Ok(Some(Def {
            name,
            loc,
            pat,
            rep,
        }))
    }

    pub fn parse_expr(&mut self, is_func: bool) -> Result<(Rc<String>, Loc, Expr), Error> {
        if !self.sc.is_token(TokenTy::Lparen)? {
            let (loc, name) = self.sc.expect_identifier()?;
            return Ok((
                Rc::new(name.clone()),
                loc,
                if is_func {
                    Expr::Fun { name, loc }
                } else {
                    Expr::Var { name, loc }
                },
            ));
        }
        let (name, loc, mut res) = self.parse_expr(true)?;

        while !self.sc.is_token(TokenTy::Rparen)? {
            let (_, _, arg) = self.parse_expr(false)?;
            res = Expr::App(Box::new(App {
                name: name.clone(),
                f: res,
                loc,
                arg,
            }))
        }

        Ok((name, loc, res))
    }
}
