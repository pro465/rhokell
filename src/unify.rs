use std::collections::HashMap;

use crate::{Def, Expr, Func};
impl Def {
    pub fn apply(&self, e: &mut Expr) -> bool {
        let mut bindings = HashMap::new();
        if !unify(&mut bindings, &self.pat, e) {
            return false;
        }
        let new_expr = substitute(&bindings, &self.rep);
        *e = new_expr;
        true
    }
}

fn unify<'a>(b: &mut HashMap<&'a str, &'a Expr>, pat: &'a Expr, e: &'a Expr) -> bool {
    match (pat, e) {
        (Expr::Var { name, .. }, _) => {
            if let Some(e2) = b.get(&name[..]) {
                *e2 == e
            } else {
                b.insert(name, e);
                true
            }
        }
        (Expr::Func(f1), Expr::Func(f2))
            if f1.name == f2.name && f1.args.len() == f2.args.len() =>
        {
            for (p, e) in f1.args.iter().zip(f2.args.iter()) {
                if !unify(b, p, e) {
                    return false;
                }
            }
            true
        }
        (Expr::Func(f1), Expr::RedFunc(f2))
            if f1.name == f2.name && f1.args.len() == f2.args.len() =>
        {
            for (p, e) in f1.args.iter().zip(f2.args.iter()) {
                if !unify(b, p, e) {
                    return false;
                }
            }
            true
        }
        _ => false,
    }
}

fn substitute(b: &HashMap<&str, &Expr>, rep: &Expr) -> Expr {
    match rep {
        Expr::Var { name, .. } => b[&name[..]].clone(),
        Expr::Func(f) => Expr::Func(Func {
            loc: f.loc,
            name: f.name.clone(),
            args: f.args.iter().map(|e| substitute(b, &e)).collect(),
        }),
        _ => unreachable!(),
    }
}
