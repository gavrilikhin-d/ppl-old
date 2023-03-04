use clap::Parser;

use driver::{commands::Execute, Args};

use miette::{bail, Result};

fn main() -> Result<()> {
    let args = Args::parse();
    if let Err(err) = args.command.execute() {
        bail!(err)
    }
    Ok(())
}
