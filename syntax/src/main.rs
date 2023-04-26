use nom::{error::convert_error, Finish};
use syntax::parsers::rule;

fn main() {
    loop {
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();
        let res = rule(&line).finish();
        if let Err(err) = res {
            println!("{}", convert_error(line.as_str(), err));
            continue;
        }

        let (_rest, (_tree, ast)) = res.unwrap();
        println!("{:?}", ast)
    }
}
