use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{Def, Expr};
impl Def {
    pub fn apply(&self, e: &mut Expr) -> bool {
        let mut bindings = HashMap::new();
        if !unify(
            &mut bindings,
            Rc::new(RefCell::new(self.pat.clone())),
            Rc::new(RefCell::new(e.clone())),
        ) {
            return false;
        }
        let new_expr = substitute(&bindings, &self.rep);
        *e = new_expr;
        true
    }
}

fn unify(
    b: &mut HashMap<String, Rc<RefCell<Expr>>>,
    pat: Rc<RefCell<Expr>>,
    e: Rc<RefCell<Expr>>,
) -> bool {
    match (&*pat.borrow(), &*e.borrow()) {
        (Expr::Var { name, .. }, _) => {
            if let Some(e2) = b.get(&name[..]) {
                *e2 == e
            } else {
                b.insert(name.clone(), e.clone());
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
                if !unify(b, p.clone(), e.clone()) {
                    return false;
                }
            }
            true
        }
        _ => false,
    }
}

fn substitute(b: &HashMap<String, Rc<RefCell<Expr>>>, rep: &Expr) -> Expr {
    match rep {
        Expr::Var { name, .. } => b[name].borrow().clone(),
        Expr::Func {
            loc, name, args, ..
        } => Expr::Func {
            loc: *loc,
            name: name.clone(),
            args: args
                .iter()
                .map(|e| Rc::new(RefCell::new(substitute(b, &e.borrow()))))
                .collect(),
            reduced: false,
        },
    }
}
