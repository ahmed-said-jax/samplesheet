mod client;
pub(super) mod config;
use anyhow::Context;
use client::GoogleSheetsClient;
use config::Config;
use reqwest::Client;
use std::str::FromStr;

use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};

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

    println!(
        "n_rows0: {}\nn_rows1: {}\nn_rows2: {}\nn_rows3: {}\n",
        data.value_ranges[0].values.len(),
        data.value_ranges[1].values.len(),
        data.value_ranges[2].values.len(),
        data.value_ranges[3].values.len()
    );

    println!("{data:?}");

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
    values: Vec<Row>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum Row {
    Full([String; 1]),
    Empty([String; 0]),
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "UPPERCASE")]
enum MajorDimension {
    Rows,
}
