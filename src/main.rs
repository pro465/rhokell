use io::Write;
use rhokell::{Alloc, DisplayWithAlloc, Rules};
use std::{fs, io};

enum Flag {
    Repl,
    Rd,
    Normal,
}

fn parse_flags(args: &[String]) -> (Flag, usize) {
    use Flag::*;
    if args.get(1) == Some(&"-r".into()) {
        (Repl, 2)
    } else if args.get(1) == Some(&"-d".into()) {
        (Rd, 2)
    } else {
        (Normal, 1)
    }
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let (ty, idx) = parse_flags(&args);
    let mut alloc = Alloc::new();

    let rules = rhokell::parse(
        &mut alloc,
        fs::read_to_string(
            fs::canonicalize(args.get(idx).unwrap_or_else(|| help()))
                .expect("could not canonicalize argument"),
        )
        .expect("could not read file"),
    );
    let rules = rules.unwrap_or_else(|e| {
        e.report();
        std::process::exit(-1);
    });
    //dbg!(&rules);
    match ty {
        Flag::Repl => repl(&mut alloc, &rules),
        _ => {
            let mut expr = rhokell::parse_expr(&mut alloc, "(main)".into()).unwrap();
            rhokell::apply(&rules, &mut expr, &mut alloc);
            if let Flag::Rd = ty {
                println!("{}", expr.to_string(&alloc));
            }
        }
    }
}

fn repl(alloc: &mut Alloc, rules: &Rules) {
    println!("welcome to rhokell v0.2.0!\ninput `q`, `quit`, or `exit` for exiting the REPL");

    let mut line = String::new();

    let mut prompt = |s| {
        print!("{s} ");
        io::stdout().flush().unwrap();
        line.clear();
        std::io::stdin()
            .read_line(&mut line)
            .expect("could not read input");
        let line = line.trim();
        if line.is_empty() {
            None
        } else {
            Some(line.to_string())
        }
    };

    while let Some(mut line) = prompt("=>") {
        if is_quit(&line) {
            break;
        }
        while line.chars().filter(|&c| c == '(').count()
            > line.chars().filter(|&c| c == ')').count()
        {
            let t = match prompt("..") {
                Some(x) if !is_quit(&x) => x,
                _ => break,
            };
            line.push_str(&t);
        }

        let expr = rhokell::parse_expr(alloc, line);
        let mut expr = match expr {
            Ok(x) => x,
            Err(e) => {
                e.report();
                continue;
            }
        };

        rhokell::apply(&rules, &mut expr, alloc);

        println!("{}", expr.to_string(alloc));
    }
}

fn help() -> ! {
    println!(
        "usage: {} [-r|-d] <filename>",
        std::env::current_exe()
            .unwrap_or_else(|_| "rhokell".into())
            .display()
    );
    std::process::exit(-1);
}

fn is_quit(s: &str) -> bool {
    ["quit", "exit", "q"].contains(&s)
}
