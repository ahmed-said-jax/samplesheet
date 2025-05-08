use std::str::FromStr;

use anyhow::Context;
use camino::Utf8PathBuf;
use clap::{Parser, Subcommand};
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
        Command::StageXenium { data_dirs } => stage_xenium_data(&xenium, &data_dirs)
            .await
            .context("failed to stage xenium data directories")?,
    }

    Ok(())
}

#[derive(Subcommand)]
enum Command {
    Samplesheet {
        fastq_paths: Vec<Utf8PathBuf>,
        #[arg(short, long, default_value_t = Utf8PathBuf::from_str("tracking-sheet").unwrap())]
        tracking_sheet_dir: Utf8PathBuf,
        #[arg(short, long, default_value_t = Utf8PathBuf::from_str("samplesheet.yaml").unwrap())]
        output_path: Utf8PathBuf,
    },
    StageXenium {
        data_dirs: Vec<Utf8PathBuf>,
    },
}

#[derive(Parser)]
struct Cli {
    #[arg(long, env = "SCBL_UTILS_CONFIG_PATH", default_value_t = Utf8PathBuf::from_str("/sc/service/.scbl-utils/config.toml").unwrap())]
    config_path: Utf8PathBuf,
    #[arg(long, env = "SCBL_UTILS_CACHE_DIR", default_value_t = Utf8PathBuf::from_str("/sc/service/.scbl-utils/cache/").unwrap())]
    cache_dir: Utf8PathBuf,
    #[command(subcommand)]
    command: Command,
}
