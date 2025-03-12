use std::str::FromStr;

use camino::Utf8PathBuf;
use clap::Parser;
use samplesheet::write_samplesheet;

fn main() -> anyhow::Result<()> {
    let Cli {
        config_path,
        fastq_paths,
        tracking_sheet_dir,
        output_path,
    } = Cli::parse();

    write_samplesheet(&config_path, &fastq_paths, &tracking_sheet_dir, &output_path)
}

#[derive(Parser)]
#[command(version, about, long_about = "")]
struct Cli {
    #[arg(short, long, default_value_t = Utf8PathBuf::from_str("/sc/service/etc/.config/samplesheet.toml").unwrap())]
    config_path: Utf8PathBuf,
    fastq_paths: Vec<Utf8PathBuf>,
    #[arg(short, long, default_value_t = Utf8PathBuf::from_str("tracking-sheet").unwrap())]
    tracking_sheet_dir: Utf8PathBuf,
    #[arg(short, long, default_value_t = Utf8PathBuf::from_str("samplesheet.yaml").unwrap())]
    output_path: Utf8PathBuf,
}
