use crate::driver::commands::New;

use super::Execute;

impl Execute for New {
    type Output = miette::Result<()>;

    /// Compile single ppl file
    fn execute(&self) -> Self::Output {
        Ok(())
    }
}
