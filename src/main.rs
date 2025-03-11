use std::{
    collections::HashMap,
    fs::{self},
    str::FromStr,
};

use anyhow::Context;
use camino::{Utf8Path, Utf8PathBuf};
use clap::Parser;
use samplesheet::generate_samplesheet;
use serde::{Deserialize, de::DeserializeOwned};

fn main() -> anyhow::Result<()> {
    let Cli {
        config_path,
        fastq_paths,
        tracking_sheet_dir,
        output_path,
    } = Cli::parse();

    let samplesheet = generate_samplesheet(&config_path, &fastq_paths, &tracking_sheet_dir)?;

    Ok(())
}

#[derive(Parser)]
#[command(version, about, long_about = "")]
struct Cli {
    #[arg(short, long, default_value_t = Utf8PathBuf::from_str("/sc/service/.config/samplesheet.toml").unwrap())]
    config_path: Utf8PathBuf,
    #[arg(short, long)]
    fastq_paths: Vec<Utf8PathBuf>,
    #[arg(short, long, default_value_t = Utf8PathBuf::from_str("tracking-sheet").unwrap())]
    tracking_sheet_dir: Utf8PathBuf,
    #[arg(short, long, default_value_t = Utf8PathBuf::from_str("samplesheet.yaml").unwrap())]
    output_path: Utf8PathBuf,
}
