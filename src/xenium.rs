mod client;
pub(super) mod config;
use anyhow::Context;
use client::GoogleSheetsClient;
use config::Config;
use reqwest::Client;
use std::str::FromStr;

use camino::Utf8PathBuf;
use serde::{Deserialize, Deserializer, Serialize};

const N_SPREADSHEET_RANGES: usize = 4;

pub async fn stage_data(data_dirs: &[Utf8PathBuf], config: &config::Config) -> anyhow::Result<()> {
    let Config {
        google_sheets_api_key,
        spreadsheet: spreadsheet_config,
        staging_dir,
    } = config;

    let client = GoogleSheetsClient::new(google_sheets_api_key).context("failed to create Google Sheets client")?;

    let data: Spreadsheet = client
        .download_spreadsheet(spreadsheet_config)
        .await
        .context("failed to download Xenium spreadsheet")?;

    Ok(())
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Spreadsheet {
    value_ranges: [ValueRange; N_SPREADSHEET_RANGES],
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ValueRange {
    range: String,
    major_dimension: MajorDimension,
    #[serde(deserialize_with = "deserialize_rows")]
    values: Vec<[String; 1]>,
}

type Rows = Vec<[String; 1]>;

fn deserialize_rows<'de, D>(deserializer: D) -> Result<Rows, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum MaybeEmptyRow {
        Full([String; 1]),
        Empty([String; 0]),
    }

    let deserialized = Vec::<MaybeEmptyRow>::deserialize(deserializer)?;

    let extract_inner = |row: MaybeEmptyRow| match row {
        MaybeEmptyRow::Empty(_) => [String::new()],
        MaybeEmptyRow::Full(data) => data,
    };

    let extracted = deserialized.into_iter().map(extract_inner).collect();

    Ok(extracted)
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "UPPERCASE")]
enum MajorDimension {
    Rows,
}
