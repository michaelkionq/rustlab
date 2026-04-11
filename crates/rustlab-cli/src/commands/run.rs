use clap::Args;
use anyhow::{Context, Result};

#[derive(Args)]
pub struct RunArgs {
    /// Path to a .r script file
    pub script: std::path::PathBuf,

    /// Profile all function calls and print a report to stderr on exit.
    /// For selective profiling, add profile(fn1, fn2) inside the script instead.
    #[arg(long)]
    pub profile: bool,
}

pub fn execute(args: RunArgs) -> Result<()> {
    let script = args.script.canonicalize()
        .with_context(|| format!("failed to resolve path {:?}", args.script))?;
    let source = std::fs::read_to_string(&script)
        .with_context(|| format!("failed to read {:?}", script))?;
    if let Some(dir) = script.parent() {
        std::env::set_current_dir(dir)
            .with_context(|| format!("failed to chdir to {:?}", dir))?;
    }
    let result = if args.profile {
        rustlab_script::run_profiled(&source)
    } else {
        rustlab_script::run(&source)
    };
    match result {
        Ok(()) => Ok(()),
        // stdin closed cleanly (e.g. audio source finished) — exit 0 silently
        Err(rustlab_script::ScriptError::AudioEof) => Ok(()),
        Err(e) => Err(anyhow::anyhow!("{}", e)),
    }
}
