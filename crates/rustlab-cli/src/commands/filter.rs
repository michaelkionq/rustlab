use anyhow::{bail, Result};
use clap::{Args, Subcommand};
use rustlab_dsp::{
    butterworth_highpass, butterworth_lowpass, fir_bandpass, fir_highpass, fir_lowpass,
    WindowFunction,
};

#[derive(Subcommand)]
pub enum FilterCommands {
    /// Design an FIR filter and print coefficients
    Fir(FirArgs),
    /// Design an IIR Butterworth filter and print coefficients
    Iir(IirArgs),
}

#[derive(Args)]
pub struct FirArgs {
    /// Number of taps
    #[arg(long, default_value = "32")]
    pub taps: usize,
    /// Cutoff frequency in Hz (for bandpass: low cutoff)
    #[arg(long)]
    pub cutoff: f64,
    /// High cutoff frequency in Hz (bandpass only)
    #[arg(long)]
    pub cutoff_high: Option<f64>,
    /// Sample rate in Hz
    #[arg(long, default_value = "44100")]
    pub sr: f64,
    /// Filter type: low, high, band
    #[arg(long, default_value = "low")]
    pub r#type: String,
    /// Window function: hann, hamming, blackman, rectangular, kaiser
    #[arg(long, default_value = "hann")]
    pub window: String,
    /// Kaiser window beta parameter
    #[arg(long)]
    pub beta: Option<f64>,
}

#[derive(Args)]
pub struct IirArgs {
    /// Filter order
    #[arg(long, default_value = "2")]
    pub order: usize,
    /// Cutoff frequency in Hz
    #[arg(long)]
    pub cutoff: f64,
    /// Sample rate in Hz
    #[arg(long, default_value = "44100")]
    pub sr: f64,
    /// Filter type: low, high
    #[arg(long, default_value = "low")]
    pub r#type: String,
}

pub fn execute(cmd: FilterCommands) -> Result<()> {
    match cmd {
        FilterCommands::Fir(args) => execute_fir(args),
        FilterCommands::Iir(args) => execute_iir(args),
    }
}

fn execute_fir(args: FirArgs) -> Result<()> {
    let window =
        WindowFunction::from_str(&args.window, args.beta).map_err(|e| anyhow::anyhow!("{}", e))?;

    let filter = match args.r#type.to_ascii_lowercase().as_str() {
        "low" | "lowpass" => fir_lowpass(args.taps, args.cutoff, args.sr, window)
            .map_err(|e| anyhow::anyhow!("{}", e))?,
        "high" | "highpass" => fir_highpass(args.taps, args.cutoff, args.sr, window)
            .map_err(|e| anyhow::anyhow!("{}", e))?,
        "band" | "bandpass" => {
            let cutoff_high = args
                .cutoff_high
                .ok_or_else(|| anyhow::anyhow!("--cutoff-high is required for bandpass filter"))?;
            fir_bandpass(args.taps, args.cutoff, cutoff_high, args.sr, window)
                .map_err(|e| anyhow::anyhow!("{}", e))?
        }
        other => bail!("unknown filter type '{}': use low, high, or band", other),
    };

    println!(
        "# FIR {} filter: {} taps, cutoff={} Hz, sr={} Hz, window={}",
        args.r#type, args.taps, args.cutoff, args.sr, args.window
    );
    for (i, c) in filter.coefficients.iter().enumerate() {
        println!("h[{:3}] = {:+.8} + {:+.8}j", i, c.re, c.im);
    }
    Ok(())
}

fn execute_iir(args: IirArgs) -> Result<()> {
    let filter = match args.r#type.to_ascii_lowercase().as_str() {
        "low" | "lowpass" => butterworth_lowpass(args.order, args.cutoff, args.sr)
            .map_err(|e| anyhow::anyhow!("{}", e))?,
        "high" | "highpass" => butterworth_highpass(args.order, args.cutoff, args.sr)
            .map_err(|e| anyhow::anyhow!("{}", e))?,
        other => bail!("unknown filter type '{}': use low or high", other),
    };

    println!(
        "# IIR Butterworth {} filter: order={}, cutoff={} Hz, sr={} Hz",
        args.r#type, args.order, args.cutoff, args.sr
    );
    println!("# Numerator coefficients (b):");
    for (i, &b) in filter.b.iter().enumerate() {
        println!("b[{:3}] = {:+.10}", i, b);
    }
    println!("# Denominator coefficients (a):");
    for (i, &a) in filter.a.iter().enumerate() {
        println!("a[{:3}] = {:+.10}", i, a);
    }
    Ok(())
}
