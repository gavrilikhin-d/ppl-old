use std::io::Write;

use syntax::{context, parsers::Parser};

fn main() {
    loop {
        print!(">>> ");
        std::io::stdout().flush().unwrap();

        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();

        let regex = context::find_rule("Regex").unwrap();

        let res = regex.parse(&line);
        if res.has_errors() {
            res.errors().for_each(|e| {
                println!(
                    "{:?}",
                    miette::Report::new_boxed(e.clone_boxed()).with_source_code(line.clone())
                )
            });
            continue;
        }
        println!("{:#?}", res.tree);
    }
}
