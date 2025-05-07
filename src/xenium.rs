mod client;
pub(super) mod config;
mod dir;
mod slide;
mod spreadsheet;

use anyhow::Context;
use camino::Utf8PathBuf;
use client::GoogleSheetsClient;
use config::Config;

const N_SPREADSHEET_RANGES: usize = 4;

pub async fn stage_data(data_dirs: &[Utf8PathBuf], config: &config::Config) -> anyhow::Result<()> {
    let Config {
        google_sheets_api_key,
        spreadsheet_spec,
        staging_dir,
    } = config;

    let client = GoogleSheetsClient::new(google_sheets_api_key).context("failed to create Google Sheets client")?;

    let data = client
        .download_spreadsheet(spreadsheet_spec)
        .await
        .context("failed to download Xenium spreadsheet")?;

    let xenium_slides = data
        .to_xenium_slides()
        .context("failed to parse spreadsheet into Xenium slides")?;

    Ok(())
}
