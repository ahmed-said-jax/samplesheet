use anyhow::Context;
use camino::Utf8PathBuf;
use clap::Parser;
use clap::Subcommand;
use scbl_utils::AppConfig;
use scbl_utils::stage_xenium_data;
use std::str::FromStr;

#[tokio::main(flavor = "current_thread")]
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
        } => todo!(),
        Command::StageXenium { data_dirs } => stage_xenium_data(&data_dirs, &xenium)
            .await
            .context("failed to stage xenium data directories")?,
    }

    Ok(())
}

#[derive(Subcommand)]
enum Command {
    Samplesheet {
        fastq_paths: Vec<Utf8PathBuf>,
        #[arg(short, long, default_value_t = Utf8PathBuf::from_str("samplesheet.yaml").unwrap())]
        output_path: Utf8PathBuf,
    },
    StageXenium {
        data_dirs: Vec<Utf8PathBuf>,
    },
}

#[derive(Parser)]
struct Cli {
    #[arg(long, env, default_value_t = Utf8PathBuf::from_str("/sc/service/.config/scbl-utils/config.toml").unwrap())]
    config_path: Utf8PathBuf,
    #[arg(long, env, default_value_t = Utf8PathBuf::from_str("/sc/service/.cache/scbl-utils/").unwrap())]
    cache_dir: Utf8PathBuf,
    #[command(subcommand)]
    command: Command,
}
