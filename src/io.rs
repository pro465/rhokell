use std::{
    io::{self, Read, Write},
    num::ParseIntError,
};

use crate::{error::Loc, parser::Expr};

pub(crate) fn input(e: &mut Expr) {
    let curr = std::io::stdin().bytes().next().transpose().unwrap();
    let src = curr.map_or("Eof".into(), |b| format!("{:02X}", b));
    *e = fun(src, e.loc());
}

pub(crate) fn output(e: &mut Expr) {
    if let Expr::App(f) = e {
        let b = decode(&f.arg).unwrap_or(0);
        let mut stdout = io::stdout();
        stdout.write_all(&[b]).unwrap();
        stdout.flush().unwrap();
        *e = fun("output".into(), e.loc());
    }
}

fn fun(name: String, loc: Loc) -> Expr {
    Expr::Fun { name, loc }
}

fn decode(e: &Expr) -> Result<u8, ParseIntError> {
    match e {
        Expr::Fun { name, .. } => u8::from_str_radix(&name, 16),
        _ => u8::from_str_radix("", 16),
    }
}
