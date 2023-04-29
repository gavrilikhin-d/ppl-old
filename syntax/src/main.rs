fn main() {
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
