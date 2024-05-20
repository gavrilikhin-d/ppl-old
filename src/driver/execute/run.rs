use crate::driver::commands::{Build, Run};

use super::Execute;

impl Execute for Run {
    type Output = miette::Result<()>;

    /// Build and run the project
    fn execute(&self) -> Self::Output {
        let exe = Build::default().execute()?;
        std::process::Command::new(exe).status().unwrap();
        Ok(())
    }
}
