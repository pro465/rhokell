use std::{
    io::{self, Read, Write},
    num::ParseIntError,
    rc::Rc,
};

use crate::{
    error::Loc,
    parser::{App, Expr},
};

type CExpr = (Expr, Rc<String>);

pub(crate) fn input(e: &mut Expr) {
    let curr = std::io::stdin().bytes().next().transpose().unwrap();
    // (byte high low)
    let src = curr.map_or(fun("EOF".into(), e.loc()), |b| {
        app(
            app(
                fun("byte".into(), e.loc()),
                fun(format!("{:X}", b >> 4), e.loc()),
                e.loc(),
            ),
            fun(format!("{:X}", b & 15), e.loc()),
            e.loc(),
        )
    });
    *e = src.0;
}

pub(crate) fn output(e: &mut Expr) {
    if let Expr::App(f) = e {
        let b = decode(&f.arg).unwrap_or(0);
        let mut stdout = io::stdout();
        stdout.write_all(&[b]).unwrap();
        stdout.flush().unwrap();
        *e = fun("output".into(), e.loc()).0;
    }
}

fn app(f: CExpr, arg: CExpr, loc: Loc) -> CExpr {
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

fn fun(name: String, loc: Loc) -> CExpr {
    let n2 = name.clone();
    (Expr::Fun { name, loc }, Rc::new(n2))
}

fn decode(e: &Expr) -> Result<u8, ParseIntError> {
    match e {
        Expr::RedApp(f) if &*f.name == "byte" => match &f.f {
            Expr::RedApp(f2) => Ok(decode_hex(&f2.arg)? << 4 | decode_hex(&f.arg)? & 15),
            _ => u8::from_str_radix("", 16),
        },
        _ => u8::from_str_radix("", 16),
    }
}

fn decode_hex(e: &Expr) -> Result<u8, ParseIntError> {
    match e {
        Expr::Fun { name, .. } => u8::from_str_radix(&name, 16),
        _ => u8::from_str_radix("", 16),
    }
}
