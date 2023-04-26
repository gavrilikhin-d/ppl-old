use syntax::parsers::rule;

fn main() {
    loop {
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();
        let res = rule(&line);
        if let Err(err) = res {
            let err = match err {
                nom::Err::Error(e) => e,
                nom::Err::Failure(e) => e,
                nom::Err::Incomplete(_) => unreachable!(),
            };
            println!("{:?}", err);
            continue;
        }

        let (_rest, (_tree, ast)) = res.unwrap();
        println!("{:?}", ast)
    }
}
