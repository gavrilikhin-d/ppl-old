use syntax::parsers::rule;

fn main() {
    loop {
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();
        println!("{:?}", rule(&line))
    }
}
