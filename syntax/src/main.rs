use nom::Finish;
use syntax::parsers::rule;

fn main() {
    loop {
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();
        let res = rule(&line).finish();
        if let Err(err) = res {
            println!("{:?}", err);
            continue;
        }

        let (_rest, (_tree, ast)) = res.unwrap();
        println!("{:?}", ast)
    }
}
