use std::fs;

use camino::{Utf8Path, Utf8PathBuf};
use serde::Deserialize;

mod samplesheet;
mod xenium;

#[derive(Deserialize)]
pub struct AppConfig {
    pub samplesheet: samplesheet::config::Config,
    pub xenium: xenium::config::Config,
}

impl AppConfig {
    pub fn read_toml_file(path: &Utf8Path) -> anyhow::Result<Self> {
        Ok(toml::from_str(&fs::read_to_string(path)?)?)
    }
}

pub async fn stage_xenium_data(
    config: &xenium::config::Config,
    data_dirs: &[Utf8PathBuf],
    skip_confirm: bool,
) -> anyhow::Result<()> {
    xenium::stage_data(config, data_dirs, skip_confirm).await
}

pub fn write_samplesheet(
    config: &samplesheet::config::Config,
    fastq_paths: &[Utf8PathBuf],
    tracking_sheet_dir: &Utf8Path,
    output_path: &Utf8Path,
) -> anyhow::Result<()> {
    samplesheet::write(config, fastq_paths, tracking_sheet_dir, output_path)
}
