use std::{
    io::{self, Read, Write},
    num::ParseIntError,
};

use crate::{
    alloc::{Alloc, Id, BYTE, EOF, OUTPUT},
    error::Loc,
    parser::{App, Expr},
};

type CExpr = (Expr, Id);

pub(crate) fn input(alloc: &mut Alloc, e: &mut Expr) {
    let curr = std::io::stdin().bytes().next().transpose().unwrap();
    // (byte high low)
    let src = curr.map_or(fun(EOF, e.loc()), |b| {
        let high = alloc.alloc_or_get(&format!("{:X}", b >> 4));
        let low = alloc.alloc_or_get(&format!("{:X}", b & 15));
        app(
            app(fun(BYTE, e.loc()), fun(high, e.loc()), e.loc()),
            fun(low, e.loc()),
            e.loc(),
        )
    });
    *e = src.0;
}

pub(crate) fn output(alloc: &Alloc, e: &mut Expr) {
    if let Expr::App(f) = e {
        let _ = decode(alloc, &f.arg).map(|b| {
            let mut stdout = io::stdout();
            stdout.write_all(&[b]).unwrap();
            stdout.flush().unwrap();
        });
        *e = fun(OUTPUT, e.loc()).0;
    }
}

fn app(f: CExpr, arg: CExpr, loc: Loc) -> CExpr {
    (
        Expr::App(Box::new(App {
            id: f.1.clone(),
            loc,
            f: f.0,
            arg: arg.0,
        })),
        f.1,
    )
}

fn fun(id: Id, loc: Loc) -> CExpr {
    let n2 = id.clone();
    (Expr::Fun { id, loc }, n2)
}

fn decode(alloc: &Alloc, e: &Expr) -> Result<u8, ParseIntError> {
    match e {
        Expr::RedApp(f) if f.id == BYTE => match &f.f {
            Expr::RedApp(f2) => {
                Ok(decode_hex(alloc, &f2.arg)? << 4 | decode_hex(alloc, &f.arg)? & 15)
            }
            _ => u8::from_str_radix("", 16),
        },
        _ => u8::from_str_radix("", 16),
    }
}

fn decode_hex(alloc: &Alloc, e: &Expr) -> Result<u8, ParseIntError> {
    match e {
        Expr::Fun { id, .. } => u8::from_str_radix(alloc.get_string(id), 16),
        _ => u8::from_str_radix("", 16),
    }
}
