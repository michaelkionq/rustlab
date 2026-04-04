use clap::Args;
use anyhow::Result;
use rustlab_dsp::WindowFunction;

#[derive(Args)]
pub struct WindowArgs {
    /// Window type: hann, hamming, blackman, rectangular, kaiser
    #[arg(long)]
    pub r#type: String,
    /// Number of samples
    #[arg(long)]
    pub length: usize,
    /// Kaiser beta (only for kaiser window)
    #[arg(long)]
    pub beta: Option<f64>,
    /// Show a plot
    #[arg(long)]
    pub plot: bool,
}

pub fn execute(args: WindowArgs) -> Result<()> {
    let window = WindowFunction::from_str(&args.r#type, args.beta)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let samples = window.generate(args.length);

    // Always print values to stdout, one per line
    for (i, &v) in samples.iter().enumerate() {
        println!("w[{:4}] = {:+.10}", i, v);
    }

    if args.plot {
        let title = format!("{} window (N={})", args.r#type, args.length);
        rustlab_plot::stem_real(&samples, &title)
            .map_err(|e| anyhow::anyhow!("{}", e))?;
    }

    Ok(())
}
