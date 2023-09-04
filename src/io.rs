use std::{
    io::{self, Read, Write},
    rc::Rc,
};

use crate::parser::Expr;

pub(crate) fn input(e: &mut Expr) {
    let curr = std::io::stdin().bytes().next().transpose().unwrap();
    *e = encode(curr);
}

fn encode(byte: Option<u8>) -> Expr {
    let b = byte.map(u16::from).unwrap_or(256);
    let mut src = format!(
        "Cons({}, Cons({}, Cons({}, Cons({}, Cons({}, Cons({}, Cons({}, Cons({}, Nil()))))))))",
        enc(b >> 7),
        enc(b >> 6),
        enc(b >> 5),
        enc(b >> 4),
        enc(b >> 3),
        enc(b >> 2),
        enc(b >> 1),
        enc(b >> 0),
    );

    if byte.is_none() {
        src = format!("Cons(T(), {})", src);
    }

    crate::parse_expr(src).unwrap_or_else(|e| {
        e.report();
        std::process::exit(-1)
    })
}

pub(crate) fn output(e: &mut Expr) {
    if let Expr::Func(f) = e {
        let args = &mut f.args;
        let b = if args.is_empty() {
            0
        } else {
            decode(0, 8, &args[0])
        };
        io::stdout().write_all(&[b]).unwrap();
        if !args.is_empty() {
            *e = args[0].clone();
        } else {
            *e = Expr::RedFunc(Rc::new(std::mem::take(f)))
        }
    }
}

fn enc(bit: u16) -> &'static str {
    if bit & 1 == 1 {
        "T()"
    } else {
        "F()"
    }
}

fn decode(res: u8, idx: u8, e: &Expr) -> u8 {
    if idx == 0 {
        return res;
    }
    match e {
        Expr::RedFunc(f) if f.name == "Cons" && f.args.len() == 2 => {
            decode(res | (dec(&f.args[0]) << idx - 1), idx - 1, &f.args[1])
        }
        _ => res,
    }
}

fn dec(x: &Expr) -> u8 {
    match x {
        Expr::RedFunc(f) if f.name == "F" => 0,
        _ => 1,
    }
}
