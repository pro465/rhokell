use std::{collections::HashMap, rc::Rc};

use crate::{Def, Expr};
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
        (
            Expr::Func { name, args, .. },
            Expr::Func {
                name: name2,
                args: args2,
                ..
            },
        ) if name2 == name && args.len() == args2.len() => {
            for (p, e) in args.iter().zip(args2.iter()) {
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
        Expr::Func { loc, name, args } => Expr::Func {
            loc: *loc,
            name: name.clone(),
            args: args.iter().map(|e| Rc::new(substitute(b, e))).collect(),
        },
    }
}
