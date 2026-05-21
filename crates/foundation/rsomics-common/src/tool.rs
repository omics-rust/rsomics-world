use std::process;

use crate::error::Result;
use crate::flags::CommonFlags;
use crate::json::ToolMeta;
use crate::runner::run;

pub trait Tool: Sized {
    fn meta() -> ToolMeta;
    fn common(&self) -> &CommonFlags;
    fn execute(self) -> Result<()>;

    fn run(self) -> process::ExitCode {
        let common = self.common().clone();
        run(&common, Self::meta(), || {
            self.execute()?;
            Ok(())
        })
    }
}
