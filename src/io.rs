use std::{
    io::{self, Read, Write},
    num::ParseIntError,
    rc::Rc,
};

use crate::{
    error::Loc,
    parser::{App, Expr},
};

pub(crate) fn input(e: &mut Expr) {
    let curr = std::io::stdin().bytes().next().transpose().unwrap();
    // (high (low))
    let src = curr.map_or(fun("Eof".into(), e.loc()), |b| {
        app(
            fun(format!("{:X}", b >> 4), e.loc()),
            fun(format!("{:X}", b & 15), e.loc()),
            e.loc(),
        )
    });
    *e = src.0;
}

pub(crate) fn output(e: &mut Expr) {
    if let Expr::App(f) = e {
        let b = decode(&f.arg, 4).unwrap_or(0);
        let mut stdout = io::stdout();
        stdout.write_all(&[b]).unwrap();
        stdout.flush().unwrap();
        *e = fun("output".into(), e.loc()).0;
    }
}

fn app(f: (Expr, Rc<String>), arg: (Expr, Rc<String>), loc: Loc) -> (Expr, Rc<String>) {
    (
        Expr::App(Box::new(App {
            name: f.1.clone(),
            loc,
            f: f.0,
            arg: arg.0,
        })),
        f.1,
    )
}

fn fun(name: String, loc: Loc) -> (Expr, Rc<String>) {
    let n2 = name.clone();
    (Expr::Fun { name, loc }, Rc::new(n2))
}

fn decode(e: &Expr, idx: u8) -> Result<u8, ParseIntError> {
    match e {
        Expr::Fun { name, .. } => u8::from_str_radix(&name, 16),
        Expr::RedApp(f) => u8::from_str_radix(&f.name, 16).and_then(|x| {
            Ok(if idx == 0 {
                x
            } else {
                (x << idx) | decode(&f.arg, 0)?
            })
        }),
        _ => u8::from_str_radix("", 16),
    }
}
