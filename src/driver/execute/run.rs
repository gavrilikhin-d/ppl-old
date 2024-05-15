use cmd_lib::run_cmd;
use miette::miette;

use crate::driver::commands::{Build, Run};

use super::Execute;

impl Execute for Run {
    type Output = miette::Result<()>;

    /// Build and run the project
    fn execute(&self) -> Self::Output {
        let exe = Build::default().execute()?;
        run_cmd!($exe).map_err(|e| miette!("{e}"))?;
        Ok(())
    }
}
