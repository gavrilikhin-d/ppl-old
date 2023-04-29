use syntax::{context, errors::TypeError, Rule};

fn main() {
    context::on_parse("Rule", |_, ast| {
        let rule = ast.downcast::<Rule>().map_err(|_| Box::new(TypeError {}))?;
        context::add_rule(*rule);
        Ok(())
    });

    loop {
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();
        break;
        // let res = rule(&line).finish();
        // if let Err(err) = res {
        //     println!("{}", convert_error(line.as_str(), err));
        //     continue;
        // }

        // let (_rest, (_tree, ast)) = res.unwrap();
        // println!("{:?}", ast)
    }
}
