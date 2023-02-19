use clap::Parser;

use driver::{commands::Execute, Args};

fn main() {
    let args = Args::parse();
    args.command.execute().unwrap();
}
