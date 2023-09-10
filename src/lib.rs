use error::{Error, ErrorTy};
use parser::{App, Def, Expr};
use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

mod error;
mod io;
mod parser;
mod token;
mod unify;

pub type Rules = HashMap<String, Vec<Def>>;

pub fn parse(src: String) -> Result<Rules, Error> {
    let scanner = token::Scanner::new(&src);
    let mut parser = parser::Parser::new(scanner);
    let mut defs: HashMap<_, Vec<Def>> = HashMap::new();
    while let Some(def) = parser.parse_def()? {
        check_closed(&def)?;
        defs.entry(def.name.clone()).or_default().push(def);
    }
    Ok(defs)
}

pub fn parse_expr(src: String) -> Result<Expr, Error> {
    let scanner = token::Scanner::new(&src);
    let mut parser = parser::Parser::new(scanner);
    let (.., e) = parser.parse_expr(false)?;
    parser.sc.expect_token(token::TokenTy::Eof)?;
    Ok(e)
}

pub fn apply(defs: &Rules, e: &mut Expr) -> bool {
    with_stacker(|| {
        let mut changed = false;

        loop {
            match e {
                Expr::App(f) => {
                    changed |= apply(defs, &mut f.f);
                    changed |= apply(defs, &mut f.arg);
                    if is_io(&f.f) {
                        io::output(e);
                    } else if !defs.contains_key(&**f.name)
                        || !defs[&**f.name].iter().any(|def| def.apply(e))
                    {
                        mark_reduced(e);
                        break changed;
                    }
                    changed = true;
                }
                Expr::Fun { name, .. } => {
                    if name == "input" {
                        io::input(e);
                    } else if !defs.contains_key(&*name)
                        || !defs[&*name].iter().any(|def| def.apply(e))
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

fn is_io(f: &Expr) -> bool {
    match f {
        Expr::Fun { name, .. } => name == "output",
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

fn check_closed(def: &Def) -> Result<(), Error> {
    let mut pat_vars = HashSet::new();
    vars(&mut pat_vars, &def.pat);
    let mut rep_vars = HashSet::new();
    vars(&mut rep_vars, &def.rep);
    let undefined: Vec<_> = rep_vars.difference(&pat_vars).cloned().collect();
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

fn vars(v: &mut HashSet<String>, e: &Expr) {
    match e {
        Expr::Var { name, .. } => {
            v.insert(name.clone());
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

fn list(it: &[String]) -> String {
    let p = it[..it.len() - 1].join(", ");
    if it.len() > 1 {
        format!("{p}, and {}", it.last().unwrap())
    } else {
        it[0].to_string()
    }
}
