use clap::Args;
use anyhow::{bail, Context, Result};
use num_complex::Complex;
use rustlab_core::CVector;
use rustlab_dsp::convolution::{convolve, overlap_add};
use ndarray::Array1;
use std::path::Path;

#[derive(Args)]
pub struct ConvolveArgs {
    /// Input signal file (CSV, one complex value per line as "re,im" or just "re")
    #[arg(long)]
    pub signal: std::path::PathBuf,
    /// Kernel file (same format)
    #[arg(long)]
    pub kernel: std::path::PathBuf,
    /// Convolution method: direct, overlap-add
    #[arg(long, default_value = "direct")]
    pub method: String,
}

/// Read a signal from a CSV file. Each line is either `re` or `re,im`.
fn read_signal(path: &Path) -> Result<CVector> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {:?}", path))?;

    let mut values: Vec<Complex<f64>> = Vec::new();
    for (line_no, line) in content.lines().enumerate() {
        let line = line.trim();
        // Skip blank lines and comment lines
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.splitn(2, ',').collect();
        match parts.len() {
            1 => {
                let re: f64 = parts[0].trim().parse().with_context(|| {
                    format!("line {}: cannot parse '{}' as f64", line_no + 1, parts[0])
                })?;
                values.push(Complex::new(re, 0.0));
            }
            2 => {
                let re: f64 = parts[0].trim().parse().with_context(|| {
                    format!("line {}: cannot parse re '{}' as f64", line_no + 1, parts[0])
                })?;
                let im: f64 = parts[1].trim().parse().with_context(|| {
                    format!("line {}: cannot parse im '{}' as f64", line_no + 1, parts[1])
                })?;
                values.push(Complex::new(re, im));
            }
            _ => bail!("line {}: unexpected format '{}'", line_no + 1, line),
        }
    }

    Ok(Array1::from_vec(values))
}

pub fn execute(args: ConvolveArgs) -> Result<()> {
    let signal = read_signal(&args.signal)?;
    let kernel = read_signal(&args.kernel)?;

    let result = match args.method.to_ascii_lowercase().as_str() {
        "direct" => {
            convolve(&signal, &kernel).map_err(|e| anyhow::anyhow!("{}", e))?
        }
        "overlap-add" | "overlap_add" | "ola" => {
            // Choose a reasonable block size: 8 × kernel length, minimum 64
            let block_size = (8 * kernel.len()).max(64);
            overlap_add(&signal, &kernel, block_size)
                .map_err(|e| anyhow::anyhow!("{}", e))?
        }
        other => bail!("unknown method '{}': use direct or overlap-add", other),
    };

    for c in result.iter() {
        println!("{:+.10},{:+.10}", c.re, c.im);
    }
    Ok(())
}
