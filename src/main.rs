use std::{
    io::{self, Read},
    str::FromStr,
};

use anyhow::{Context, bail};
use camino::Utf8PathBuf;
use clap::{Parser, Subcommand};
use console::Term;
use scbl_utils::{AppConfig, stage_xenium_data, write_samplesheet};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let Cli {
        config_path,
        cache_dir,
        command,
    } = Cli::parse();

    let AppConfig { samplesheet, xenium } =
        AppConfig::read_toml_file(&config_path).context("failed to read scbl-utils configuration")?;

    match command {
        Command::Samplesheet {
            fastq_paths,
            output_path,
            tracking_sheet_dir,
        } => write_samplesheet(&samplesheet, &fastq_paths, &tracking_sheet_dir, &output_path)?,
        Command::StageXenium { data_dirs, yes } => stage_xenium_data(&xenium, &data_dirs, yes)
            .await
            .context("failed to stage xenium data directories")?,
    }

    Ok(())
}

#[derive(Subcommand)]
enum Command {
    /// Generate a new samplesheet for use with the nf-tenx pipeline
    Samplesheet {
        /// The fastq files from which to generate a samplesheet. Note that this expects fastq *files*, not directories. If you want to pass an entire directory's worth of files, just use globs like `/path/to/fastq-dir/*`
        fastq_paths: Vec<Utf8PathBuf>,
        /// The directory in which the tracking sheet is downloaded. Soon to be deprecated in favor of automatic fetching from Microsoft OneDrive/SharePoint
        #[arg(short, long, default_value_t = Utf8PathBuf::from_str("tracking-sheet").unwrap())]
        tracking_sheet_dir: Utf8PathBuf,
        /// The path at which to write the resulting samplesheet
        #[arg(short, long, default_value_t = Utf8PathBuf::from_str("samplesheet.yaml").unwrap())]
        output_path: Utf8PathBuf,
    },
    /// Move the outputs of a Xenium run into the staging directory for delivery
    StageXenium {
        /// The data directories produced by the instrument
        data_dirs: Vec<Utf8PathBuf>,
        /// Move the files without confirmation (useful for batch jobs or scripts)
        #[arg(short, long, default_value_t)]
        yes: bool,
    },
}

/// A command-line utility to aid in data-processing and delivery at the Single Cell Biology Laboratory at the Jackson Laboratory
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Path to the scbl-utils configuration file. See https://github.com/ahmed-said-jax/scbl-utils/blob/main/config.sample.toml for an almost-complete configuration that works for elion.
    #[arg(long, env = "SCBL_UTILS_CONFIG_PATH", default_value_t = Utf8PathBuf::from_str("/sc/service/.config/scbl-utils/config.toml").unwrap())]
    config_path: Utf8PathBuf,
    /// Path to the scbl-utils cache directory
    #[arg(long, env = "SCBL_UTILS_CACHE_DIR", default_value_t = Utf8PathBuf::from_str("/sc/service/.cache/scbl-utils/").unwrap())]
    cache_dir: Utf8PathBuf,
    /// Command
    #[command(subcommand)]
    command: Command,
}
