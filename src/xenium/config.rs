use std::{collections::HashMap, str::FromStr};

use anyhow::anyhow;
use camino::Utf8PathBuf;
use reqwest::Url;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub(super) google_sheets_api_key: String,
    pub(super) spreadsheet_spec: SpreadsheetSpecification,
    pub(super) staging_dir_spec: StagingDirSpecification,
}

#[derive(Deserialize)]
pub(super) struct SpreadsheetSpecification {
    id: String,
    range: String,
    pub(super) slide_id_col_idx: u8,
    pub(super) slide_name_col_idx: u8,
    pub(super) run_id_col_idx: u8,
    pub(super) lab_name_col_idx: u8,
}

impl SpreadsheetSpecification {
    pub(super) fn to_url(&self) -> anyhow::Result<Url> {
        let Self { id, range, .. } = self;

        let url = format!("https://sheets.googleapis.com/v4/spreadsheets/{id}/values/{range}");

        Ok(Url::from_str(&url)?)
    }
}

#[derive(Deserialize)]
pub(super) struct StagingDirSpecification {
    root: Utf8PathBuf,
    lab_dirs: HashMap<String, Utf8PathBuf>,
}

impl StagingDirSpecification {
    pub(super) fn lab_staging_dir(&self, lab_name: &str) -> anyhow::Result<Utf8PathBuf> {
        let Self {
            root,
            lab_dirs: lab_name_map,
        } = self;

        lab_name_map
            .get(lab_name)
            .map(|p| root.join(p))
            .ok_or(anyhow!("failed to find staging directory for {lab_name}"))
    }
}
