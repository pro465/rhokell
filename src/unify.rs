use std::{collections::HashMap, rc::Rc};

use crate::{App, Def, Expr};

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
        (Expr::App(f1), Expr::App(f2)) if f1.name == f2.name => {
            unify(b, &f1.f, &f2.f) && unify(b, &f1.arg, &f2.arg)
        }
        (Expr::App(f1), Expr::RedApp(f2)) if f1.name == f2.name => {
            unify(b, &f1.f, &f2.f) && unify(b, &f1.arg, &f2.arg)
        }

        (Expr::Fun { name, .. }, Expr::Fun { name: name2, .. }) => name == name2,
        _ => false,
    }
}

fn substitute(b: &HashMap<&str, &Expr>, rep: &Expr) -> Expr {
    match rep {
        Expr::Var { name, .. } => b[&name[..]].clone(),
        Expr::Fun { name, .. } if b.contains_key(&**name) => b[&name[..]].clone(),
        Expr::App(f) => {
            let res = substitute(b, &f.f);
            Expr::App(Box::new(App {
                name: get_name(&res),
                f: res,
                loc: f.loc,
                arg: substitute(b, &f.arg),
            }))
        }
        Expr::Fun { .. } => rep.clone(),
        _ => unreachable!(),
    }
}

fn get_name(a: &Expr) -> Rc<String> {
    match a {
        Expr::Var { name, .. } | Expr::Fun { name, .. } => Rc::new(name.clone()),
        Expr::App(f) => f.name.clone(),
        Expr::RedApp(f) => f.name.clone(),
    }
}
