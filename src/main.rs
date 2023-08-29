use io::Write;
use std::{fs, io};

fn main() {
    let rules = rhokell::parse(
        fs::read_to_string(
            fs::canonicalize(std::env::args().nth(1).unwrap_or_else(|| help()))
                .expect("could not canonicalize argument"),
        )
        .expect("could not read file"),
    );
    let rules = match rules {
        Ok(x) => x,
        Err(e) => {
            e.report();
            std::process::exit(-1);
        }
    };
    //dbg!(&rules);

    println!("welcome to rhokell v0.1.0!\ninput `q`, `quit`, or `exit` for exiting the REPL");
    print!("==> ");
    io::stdout().flush().unwrap();

    for line in std::io::stdin()
        .lines()
        .map(|e| e.expect("could not read input"))
    {
        if ["quit", "exit", "q"].contains(&&line[..]) {
            break;
        }
        let expr = rhokell::parse_expr(line);
        let mut expr = match expr {
            Ok(x) => x,
            Err(e) => {
                e.report();
                print!("==> ");
                io::stdout().flush().unwrap();
                continue;
            }
        };

        rhokell::apply(&rules, &mut expr);

        println!("{}", expr);

        print!("==> ");
        io::stdout().flush().unwrap();
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
