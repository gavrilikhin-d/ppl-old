use std::{any::Any, io::Write};

use syntax::{context, errors::TypeError, parsers, Rule};

fn main() {
    let rules = parsers::create_default_rules();
    rules.into_iter().for_each(|rule| context::add_rule(rule));

    context::on_parse("Rule", |_, ast| {
        let rule = ast.downcast_ref::<Rule>().ok_or_else(|| TypeError {})?;
        context::add_rule(rule.clone());
        Ok(ast)
    });

    context::on_parse("Regex", |_, ast| {
        Ok(ast
            .downcast::<Vec<Box<dyn Any>>>()
            .unwrap()
            .into_iter()
            .map(|b| b.downcast::<String>().unwrap())
            .next()
            .unwrap())
    });

    loop {
        print!(">>> ");
        std::io::stdout().flush().unwrap();

        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();

        let regex = context::find_rule("Regex").unwrap();

        let res = regex.lock().unwrap().parse(&line).finish();
        if let Err(err) = res {
            println!("{}", err);
            continue;
        }

        let (_rest, (_tree, ast)) = res.unwrap();
        println!("{}", ast.downcast::<String>().unwrap());
    }
}
