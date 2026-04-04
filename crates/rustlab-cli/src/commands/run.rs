use clap::Args;
use anyhow::{Context, Result};

#[derive(Args)]
pub struct RunArgs {
    /// Path to a .r script file
    pub script: std::path::PathBuf,
}

pub fn execute(args: RunArgs) -> Result<()> {
    let source = std::fs::read_to_string(&args.script)
        .with_context(|| format!("failed to read {:?}", args.script))?;
    rustlab_script::run(&source).map_err(|e| anyhow::anyhow!("{}", e))
}
