use anyhow::{bail, Context, Result};
use clap::Args;
use ndarray::Array1;
use num_complex::Complex;
use std::path::Path;

#[derive(Args)]
pub struct PlotArgs {
    /// Input file (CSV, one value per line: "re" or "re,im")
    #[arg(long)]
    pub input: std::path::PathBuf,
    /// Plot title
    #[arg(long, default_value = "Signal")]
    pub title: String,
    /// Plot type: line, stem
    #[arg(long, default_value = "line")]
    pub r#type: String,
}

/// Parse a signal file, returning (real_values, is_complex).
/// Lines may be `re` or `re,im`; blank lines and `#` comments are skipped.
fn read_signal(path: &Path) -> Result<(Vec<Complex<f64>>, bool)> {
    let content =
        std::fs::read_to_string(path).with_context(|| format!("failed to read {:?}", path))?;

    let mut values: Vec<Complex<f64>> = Vec::new();
    let mut any_complex = false;

    for (line_no, line) in content.lines().enumerate() {
        let line = line.trim();
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
                    format!(
                        "line {}: cannot parse re '{}' as f64",
                        line_no + 1,
                        parts[0]
                    )
                })?;
                let im: f64 = parts[1].trim().parse().with_context(|| {
                    format!(
                        "line {}: cannot parse im '{}' as f64",
                        line_no + 1,
                        parts[1]
                    )
                })?;
                if im != 0.0 {
                    any_complex = true;
                }
                values.push(Complex::new(re, im));
            }
            _ => bail!("line {}: unexpected format '{}'", line_no + 1, line),
        }
    }

    Ok((values, any_complex))
}

pub fn execute(args: PlotArgs) -> Result<()> {
    let (values, is_complex) = read_signal(&args.input)?;

    match args.r#type.to_ascii_lowercase().as_str() {
        "line" => {
            if is_complex {
                let cv = Array1::from_vec(values);
                rustlab_plot::plot_complex(&cv, &args.title)
                    .map_err(|e| anyhow::anyhow!("{}", e))?;
            } else {
                let rv = Array1::from_vec(values.iter().map(|c| c.re).collect());
                rustlab_plot::plot_real(&rv, &args.title).map_err(|e| anyhow::anyhow!("{}", e))?;
            }
        }
        "stem" => {
            if is_complex {
                // stem only supports real; use magnitude
                let rv = Array1::from_vec(values.iter().map(|c| c.norm()).collect());
                let title = format!("{} (magnitude)", args.title);
                rustlab_plot::stem_real(&rv, &title).map_err(|e| anyhow::anyhow!("{}", e))?;
            } else {
                let rv = Array1::from_vec(values.iter().map(|c| c.re).collect());
                rustlab_plot::stem_real(&rv, &args.title).map_err(|e| anyhow::anyhow!("{}", e))?;
            }
        }
        other => bail!("unknown plot type '{}': use line or stem", other),
    }

    Ok(())
}
