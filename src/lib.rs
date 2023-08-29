use error::{Error, ErrorTy};
use parser::{Def, Expr};
use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

mod error;
mod parser;
mod token;
pub mod unify;

pub fn parse(src: String) -> Result<HashMap<String, Vec<Def>>, Error> {
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
    let e = parser.parse_expr::<false>()?;
    parser.sc.expect_token(token::TokenTy::Eof)?;
    Ok(e)
}

pub fn apply(defs: &HashMap<String, Vec<Def>>, e: &mut Expr) -> bool {
    stacker::maybe_grow(32 * 1024, 1024 * 1024, || {
        let mut changed = false;

        loop {
            match &mut *e {
                Expr::Func { name, args, .. } => {
                    for a in args.iter_mut().map(Rc::make_mut) {
                        changed |= apply(defs, a);
                    }
                    if !defs.contains_key(name) || !defs[name].iter().any(|def| def.apply(e)) {
                        break changed;
                    }
                    changed = true;
                }
                Expr::Var { .. } => break changed,
            }
        }
    })
}

fn check_closed(def: &Def) -> Result<(), Error> {
    let mut pat_vars = HashSet::new();
    vars(&mut pat_vars, &def.pat);
    let mut rep_vars = HashSet::new();
    vars(&mut rep_vars, &def.rep);
    let undefined: Vec<_> = rep_vars.difference(&pat_vars).copied().collect();
    if !undefined.is_empty() {
        Err(Error {
            loc: def.rep.loc(),
            ty: ErrorTy::CExprError,
            desc: format!("undefined varables: {}", list(&undefined)),
        })
    } else {
        Ok(())
    }
}

fn vars<'a>(v: &mut HashSet<&'a str>, e: &'a Expr) {
    match e {
        Expr::Var { name, .. } => {
            v.insert(name);
        }
        Expr::Func { args, .. } => args.iter().for_each(|i| vars(v, i)),
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
