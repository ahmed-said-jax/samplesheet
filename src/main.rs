use anyhow::{anyhow, Context};
use camino::{Utf8Path, Utf8PathBuf};
use clap::Parser;
use serde::{de::DeserializeOwned, Deserialize};
use std::{collections::HashMap, ffi::OsString, fs::{self, read_dir, File}, result, str::FromStr};

fn main() -> anyhow::Result<()> {
    let Cli {config_path, fastq_paths, tracking_sheet_dir} = Cli::parse();

    let config = Config::from_path(&config_path).context("failed to read configuration file")?;

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
}

#[derive(Deserialize)]
struct Config {
    reference_paths: HashMap<String, String>,
    chemistry_tool: HashMap<String, String>,
    nf_tenx_repo: String,
    probe_set_paths: HashMap<String, String>
}

impl Config {
    fn from_path(path: &Utf8Path) -> anyhow::Result<Self> {
        let contents = fs::read_to_string(&path)?;
        Ok(toml::from_str(&contents)?)
    }
}

struct TrackingSheet {
    suspensions: Vec<Suspension>,
    multiplexed_suspensions: Vec<MultiplexedSuspension>,
    gems: Vec<Gems>,
    gems_suspensions: Vec<GemsSuspensions>,
    libraries: Vec<Library>,
}

#[derive(Deserialize)]
struct Suspension {

}
impl FromTrackingSheetDir for Suspension {
    fn filename() -> &'static str {
        "Chromium(Suspensions).csv"
    }
}

#[derive(Deserialize)]
struct MultiplexedSuspension {

}
impl FromTrackingSheetDir for MultiplexedSuspension {
    fn filename() -> &'static str {
        "Chromium(Multiplexed Suspensions).csv"
    }
}

#[derive(Deserialize)]
struct Gems {

}
impl FromTrackingSheetDir for Gems {
    fn filename() -> &'static str {
        "Chromium(GEMs).csv"
    }
}

#[derive(Deserialize)]
struct GemsSuspensions {

}

impl FromTrackingSheetDir for GemsSuspensions {
    fn filename() -> &'static str {
        "Chromium(GEMs-Suspensions).csv"
    }
}

#[derive(Deserialize)]
struct Library {

}

impl FromTrackingSheetDir for Library {
    fn filename() -> &'static str {
        "Chromium(Libraries).csv"
    }
}

trait FromTrackingSheetDir: Sized + DeserializeOwned{
    fn from_csv(path: &Utf8Path) -> anyhow::Result<Vec<Self>> {
        let mut reader = csv::Reader::from_path(path)?;
        let records: anyhow::Result<Vec<_>, _> = reader.deserialize().collect();

        Ok(records?)
    }

    fn filename() -> &'static str;

    fn from_tracking_sheet_dir(dir: &Utf8Path) -> anyhow::Result<Vec<Self>> {
        let path = dir.join(Self::filename());

        Self::from_csv(&path)
    }
}

impl TrackingSheet {
    fn from_dir(path: &Utf8Path) -> anyhow::Result<Self> {
        Ok(Self {
            suspensions: Suspension::from_tracking_sheet_dir(path)?,
            multiplexed_suspensions: MultiplexedSuspension::from_tracking_sheet_dir(path)?,
            gems: Gems::from_tracking_sheet_dir(path)?,
            gems_suspensions: GemsSuspensions::from_tracking_sheet_dir(path)?,
            libraries: Library::from_tracking_sheet_dir(path)?
        })
    }
}
