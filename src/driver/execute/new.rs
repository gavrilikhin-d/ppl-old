use cmd_lib::run_cmd;
use miette::miette;

use crate::driver::commands::New;

use super::Execute;

impl Execute for New {
    type Output = miette::Result<()>;

    /// Compile single ppl file
    fn execute(&self) -> Self::Output {
        const TEMPLATE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/driver/template");
        let package = &self.package;
        run_cmd!(
            cp -r $TEMPLATE $package
        )
        .map_err(|e| miette!("{e}"))?;
        Ok(())
    }
}
