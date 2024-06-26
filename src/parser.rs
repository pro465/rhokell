use std::rc::Rc;

use crate::{
    alloc::{Alloc, DisplayWithAlloc, Id},
    error::{Error, ErrorTy, Loc},
    token::{Scanner, TokenTy},
};

#[derive(Clone, Debug)]
pub struct Def {
    pub id: Id,
    pub loc: Loc,
    pub(crate) pat: Expr,
    pub(crate) rep: Expr,
}

#[derive(Clone, Debug)]
pub enum Expr {
    Fun { id: Id, loc: Loc },
    Var { id: Id, loc: Loc },
    // reduced function
    RedApp(Rc<App>),
    // unreduced function
    App(Box<App>),
}

#[derive(Clone, Debug, Default)]
pub struct App {
    pub(crate) id: Id,
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

    fn display_internal(&self, alloc: &Alloc, s: &mut String, parens: bool) {
        crate::with_stacker(|| {
            let parens = parens && !matches!(self, Expr::Var { .. });
            if parens {
                s.push('(');
            }

            match self {
                Expr::RedApp(fun) => {
                    let App { f, arg, .. } = &**fun;
                    f.display_internal(alloc, s, false);
                    s.push(' ');
                    arg.display_internal(alloc, s, true)
                }
                Expr::App(fun) => {
                    let App { f, arg, .. } = &**fun;
                    f.display_internal(alloc, s, false);
                    s.push(' ');
                    arg.display_internal(alloc, s, true)
                }
                Expr::Var { id, .. } => s.push_str(alloc.get_string(id)),
                Expr::Fun { id, .. } => s.push_str(alloc.get_string(id)),
            }
            if parens {
                s.push(')');
            }
        })
    }
}

impl Default for Expr {
    fn default() -> Self {
        Expr::Var {
            id: Id::default(),
            loc: Loc::default(),
        }
    }
}

impl PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Expr::Var { id, .. }, Expr::Var { id: id2, .. }) => id == id2,
            (Expr::Fun { id, .. }, Expr::Fun { id: id2, .. }) => id == id2,
            (Expr::RedApp(f1), Expr::RedApp(f2)) => f1 == f2,
            (Expr::App(f1), Expr::App(f2)) => f1 == f2,
            _ => false,
        }
    }
}

impl DisplayWithAlloc for Expr {
    fn display(&self, alloc: &Alloc, s: &mut String) {
        self.display_internal(alloc, s, true)
    }
}

impl PartialEq for App {
    fn eq(&self, other: &Self) -> bool {
        self.f == other.f && self.arg == other.arg
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
    pub fn parse_def(&mut self, alloc: &mut Alloc) -> Result<Option<Def>, Error> {
        if self.sc.peek(alloc)?.ty() == TokenTy::Eof {
            return Ok(None);
        }
        let (id, loc, pat) = self.parse_expr(alloc, false)?;
        if let Expr::Var { .. } = pat {
            return Err(Error {
                loc,
                ty: ErrorTy::SyntaxError,
                desc: "bare variables are not allowed".into(),
            });
        }
        self.sc.expect_token(alloc, TokenTy::Equal)?;
        let (_, _, rep) = self.parse_expr(alloc, false)?;
        self.sc.expect_token(alloc, TokenTy::Semi)?;

        Ok(Some(Def { id, loc, pat, rep }))
    }

    pub fn parse_expr(
        &mut self,
        alloc: &mut Alloc,
        is_func: bool,
    ) -> Result<(Id, Loc, Expr), Error> {
        if !self.sc.is_token(alloc, TokenTy::Lparen)? {
            let (loc, id) = self.sc.expect_identifier(alloc)?;
            return Ok((
                id.clone(),
                loc,
                if is_func {
                    Expr::Fun { id, loc }
                } else {
                    Expr::Var { id, loc }
                },
            ));
        }
        let (id, loc, mut res) = self.parse_expr(alloc, true)?;

        while !self.sc.is_token(alloc, TokenTy::Rparen)? {
            let (_, _, arg) = self.parse_expr(alloc, false)?;
            res = Expr::App(Box::new(App {
                id: id.clone(),
                f: res,
                loc,
                arg,
            }))
        }

        Ok((id, loc, res))
    }
}
