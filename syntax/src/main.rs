use std::io::Write;

use syntax::{parsers::Parser, Context};

fn main() {
    miette::set_panic_hook();

    let mut context = Context::default();
    loop {
        print!(">>> ");
        std::io::stdout().flush().unwrap();

        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();

        let regex = context.find_rule("Rule").unwrap();

        let res = regex.parse(&line, &mut context);
        if res.has_errors() {
            res.errors().for_each(|e| {
                println!(
                    "{:?}",
                    miette::Report::new(e.clone()).with_source_code(line.clone())
                )
            });
            continue;
        }
        println!("{}", serde_json::to_string_pretty(&res.ast).unwrap());
    }
}
