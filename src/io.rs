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
    let src = byte.map_or("Eof()".into(), |b| {
        let enc = |idx| ["F", "T"][(b >> idx) as usize & 1];
        format!(
            "{} {} {} {} {} {} {} {}()",
            enc(7),
            enc(6),
            enc(5),
            enc(4),
            enc(3),
            enc(2),
            enc(1),
            enc(0),
        )
    });

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

fn decode(res: u8, idx: u8, e: &Expr) -> u8 {
    if idx == 0 {
        return res;
    }
    match e {
        Expr::RedFunc(f) if f.name == "T" => {
            let res = res | (1 << idx - 1);
            if let Some(x) = f.args.get(0) {
                decode(res, idx - 1, x)
            } else {
                res >> idx - 1
            }
        }
        Expr::RedFunc(f) if f.name == "F" => {
            if let Some(x) = f.args.get(0) {
                decode(res, idx - 1, x)
            } else {
                res >> idx - 1
            }
        }
        _ => res >> idx - 1,
    }
}
