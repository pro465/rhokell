use std::{
    io::{self, Read, Write},
    num::ParseIntError,
    rc::Rc,
};

use crate::parser::Expr;

pub(crate) fn input(e: &mut Expr) {
    let curr = std::io::stdin().bytes().next().transpose().unwrap();
    *e = encode(curr);
}

fn encode(byte: Option<u8>) -> Expr {
    let src = byte.map_or("Eof()".into(), |b| format!("{:02X}()", b));

    crate::parse_expr(src).unwrap_or_else(|e| {
        e.report();
        std::process::exit(-1)
    })
}

pub(crate) fn output(e: &mut Expr) {
    if let Expr::Func(f) = e {
        let args = &mut f.args;
        let b = decode(args.get(0)).unwrap_or(0);
        io::stdout().write_all(&[b]).unwrap();
        if !args.is_empty() {
            *e = args[0].clone();
        } else {
            *e = Expr::RedFunc(Rc::new(std::mem::take(f)))
        }
    }
}

fn decode(e: Option<&Expr>) -> Result<u8, ParseIntError> {
    match e {
        Some(Expr::RedFunc(f)) => u8::from_str_radix(&f.name, 16),
        _ => u8::from_str_radix("", 16),
    }
}
