use io::Write;
use rhokell::Rules;
use std::{fs, io};

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let (idx, is_repl) = if args.get(1) == Some(&"-r".into()) {
        (2, true)
    } else {
        (1, false)
    };

    let rules = rhokell::parse(
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
    if is_repl {
        repl(&rules);
    } else {
        let mut expr = rhokell::parse_expr("main()".into()).unwrap();
        rhokell::apply(&rules, &mut expr);
    }
}

fn repl(rules: &Rules) {
    println!("welcome to rhokell v0.1.0!\ninput `q`, `quit`, or `exit` for exiting the REPL");

    let mut lines = std::io::stdin()
        .lines()
        .map(|e| e.expect("could not read input"));

    let mut prompt = |s| {
        print!("{s} ");
        io::stdout().flush().unwrap();
        lines.next()
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

        let expr = rhokell::parse_expr(line);
        let mut expr = match expr {
            Ok(x) => x,
            Err(e) => {
                e.report();
                continue;
            }
        };

        rhokell::apply(&rules, &mut expr);

        println!("{}", expr);
    }
}

fn help() -> ! {
    println!(
        "usage: {} <filename>",
        std::env::current_exe()
            .unwrap_or_else(|_| "rhokell".into())
            .display()
    );
    std::process::exit(-1);
}

fn is_quit(s: &str) -> bool {
    ["quit", "exit", "q"].contains(&s)
}
