mod client;
pub(super) mod config;
mod dir;
mod slide;
mod spreadsheet;

use anyhow::Context;
use camino::Utf8PathBuf;
use client::GoogleSheetsClient;
use config::{Config, SpreadsheetSpecification};
use dir::ParsedDataDir;
use itertools::Itertools;

const N_FIELDS: usize = 4;

pub async fn stage_data(data_dirs: &[Utf8PathBuf], config: &config::Config) -> anyhow::Result<()> {
    let Config {
        google_sheets_api_key,
        spreadsheet_spec,
        staging_dir_spec,
    } = config;

    let parsed_data_dirs: Vec<_> = data_dirs
        .iter()
        .map(ParsedDataDir::from_dir)
        .try_collect()
        .context("failing way up here")?;

    let client = GoogleSheetsClient::new(google_sheets_api_key).context("failed to create Google Sheets client")?;

    let data = client
        .download_spreadsheet(spreadsheet_spec)
        .await
        .context("failed to download Xenium spreadsheet")?;

    let xenium_slides = data
        .to_xenium_slides(spreadsheet_spec)
        .context("failed to parse spreadsheet into Xenium slides")?;

    let renamings: Vec<_> = parsed_data_dirs
        .iter()
        .map(|d| {
            d.construct_new_subdir_names(&xenium_slides, staging_dir_spec)
                .context("failing here")
        })
        .try_collect()?;

    println!(
        "For each Xenium data directory, the path renaming will be displayed. Press 'y' to confirm the move. Otherwise, press any other key."
    );
    let mut move_futures = Vec::new();
    for renaming in &renamings {
        for (old_path, new_path) in renaming {
            move_futures.push(dir::rename(old_path, new_path));
        }
    }

    futures::future::try_join_all(move_futures).await?;

    Ok(())
}
