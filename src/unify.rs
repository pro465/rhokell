use std::collections::HashMap;

use crate::{alloc::Id, App, Def, Expr};

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

fn unify<'a>(b: &mut HashMap<&'a Id, &'a Expr>, pat: &'a Expr, e: &'a Expr) -> bool {
    match (pat, e) {
        (Expr::Var { id, .. }, _) => {
            if let Some(e2) = b.get(&id) {
                *e2 == e
            } else {
                b.insert(id, e);
                true
            }
        }
        (Expr::App(f1), Expr::App(f2)) if f1.id == f2.id => {
            unify(b, &f1.f, &f2.f) && unify(b, &f1.arg, &f2.arg)
        }
        (Expr::App(f1), Expr::RedApp(f2)) if f1.id == f2.id => {
            unify(b, &f1.f, &f2.f) && unify(b, &f1.arg, &f2.arg)
        }

        (Expr::Fun { id, .. }, Expr::Fun { id: id2, .. }) => id == id2,
        _ => false,
    }
}

fn substitute(b: &HashMap<&Id, &Expr>, rep: &Expr) -> Expr {
    match rep {
        Expr::Var { id, .. } => b[&id].clone(),
        Expr::Fun { id, .. } if b.contains_key(&*id) => b[&id].clone(),
        Expr::App(f) => {
            let res = substitute(b, &f.f);
            Expr::App(Box::new(App {
                id: get_id(&res),
                f: res,
                loc: f.loc,
                arg: substitute(b, &f.arg),
            }))
        }
        Expr::Fun { .. } => rep.clone(),
        _ => unreachable!(),
    }
}

fn get_id(a: &Expr) -> Id {
    match a {
        Expr::Var { id, .. } | Expr::Fun { id, .. } => id.clone(),
        Expr::App(f) => f.id.clone(),
        Expr::RedApp(f) => f.id.clone(),
    }
}
