use std::fs;

use camino::{Utf8Path, Utf8PathBuf};
use serde::Deserialize;

mod samplesheet;
mod xenium;

#[derive(Deserialize)]
pub struct AppConfig {
    // placeholder
    pub samplesheet: Option<String>,
    pub xenium: xenium::config::Config,
}

impl AppConfig {
    pub fn read_toml_file(path: &Utf8Path) -> anyhow::Result<Self> {
        Ok(toml::from_str(&fs::read_to_string(path)?)?)
    }
}

pub async fn stage_xenium_data(data_dirs: &[Utf8PathBuf], config: &xenium::config::Config) -> anyhow::Result<()> {
    xenium::stage_data(data_dirs, config).await
}
