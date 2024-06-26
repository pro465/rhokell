use error::{Error, ErrorTy};
use parser::{App, Def, Expr};
use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

mod alloc;
mod error;
mod io;
mod parser;
mod token;
mod unify;

use alloc::Id;
pub use alloc::{Alloc, DisplayWithAlloc};

pub type Rules = HashMap<Id, Vec<Def>>;

pub fn parse(alloc: &mut Alloc, src: String) -> Result<Rules, Error> {
    let scanner = token::Scanner::new(&src);
    let mut parser = parser::Parser::new(scanner);
    let mut defs: HashMap<_, Vec<Def>> = HashMap::new();
    while let Some(def) = parser.parse_def(alloc)? {
        check_closed(alloc, &def)?;
        defs.entry(def.id.clone()).or_default().push(def);
    }
    Ok(defs)
}

pub fn parse_expr(alloc: &mut Alloc, src: String) -> Result<Expr, Error> {
    let scanner = token::Scanner::new(&src);
    let mut parser = parser::Parser::new(scanner);
    let (.., e) = parser.parse_expr(alloc, false)?;
    parser.sc.expect_token(alloc, token::TokenTy::Eof)?;
    Ok(e)
}

pub fn apply(defs: &Rules, e: &mut Expr, alloc: &mut Alloc) -> bool {
    with_stacker(|| {
        let mut changed = false;

        loop {
            match e {
                Expr::App(f) => {
                    changed |= apply(defs, &mut f.f, alloc);
                    changed |= apply(defs, &mut f.arg, alloc);
                    f.id = get_id(&f.f);
                    if is_io(&f.f) {
                        io::output(alloc, e);
                    } else if !defs.contains_key(&f.id)
                        || !defs[&f.id].iter().any(|def| def.apply(e))
                    {
                        mark_reduced(e);
                        break changed;
                    }
                    changed = true;
                }
                Expr::Fun { id, .. } => {
                    let f_id = &*id;
                    if &alloc::INPUT == id {
                        io::input(alloc, e);
                    } else if !defs.contains_key(&*f_id)
                        || !defs[&*id].iter().any(|def| def.apply(e))
                    {
                        break changed;
                    }

                    changed = true;
                }
                _ => break changed,
            }
        }
    })
}

fn get_id(e: &Expr) -> Id {
    match e {
        Expr::App(f) => f.id.clone(),
        Expr::RedApp(f) => f.id.clone(),
        Expr::Fun { id, .. } => id.clone(),
        Expr::Var { .. } => unreachable!(),
    }
}

fn is_io(f: &Expr) -> bool {
    match f {
        Expr::Fun { id, .. } => &alloc::OUTPUT == id,
        _ => false,
    }
}

pub fn with_stacker<R>(f: impl FnOnce() -> R) -> R {
    stacker::maybe_grow(32 * 1024, 1024 * 1024, f)
}

fn mark_reduced(e: &mut Expr) {
    if let Expr::App(f) = e {
        *e = Expr::RedApp(Rc::new(std::mem::take(f)))
    }
}

fn check_closed(alloc: &Alloc, def: &Def) -> Result<(), Error> {
    let mut pat_vars = HashSet::new();
    vars(&mut pat_vars, &def.pat);
    let mut rep_vars = HashSet::new();
    vars(&mut rep_vars, &def.rep);
    let undefined: Vec<_> = rep_vars
        .difference(&pat_vars)
        .map(|i| alloc.get_string(i))
        .collect();
    if !undefined.is_empty() {
        Err(Error {
            loc: def.rep.loc(),
            ty: ErrorTy::CExprError,
            desc: format!("undefined variables: {}", list(&undefined)),
        })
    } else {
        Ok(())
    }
}

fn vars(v: &mut HashSet<Id>, e: &Expr) {
    match e {
        Expr::Var { id, .. } => {
            v.insert(id.clone());
        }

        Expr::App(a) => {
            vars(v, &a.f);
            vars(v, &a.arg);
        }

        Expr::RedApp(a) => {
            vars(v, &a.f);
            vars(v, &a.arg);
        }

        Expr::Fun { .. } => {}
    }
}

fn list(it: &[&str]) -> String {
    let p = it[..it.len() - 1].join(", ");
    if it.len() > 1 {
        format!("{p}, and {}", it.last().unwrap())
    } else {
        it[0].to_string()
    }
}
