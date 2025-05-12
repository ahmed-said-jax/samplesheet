mod client;
pub(super) mod config;
mod dir;
mod slide;
mod spreadsheet;

use anyhow::Context;
use camino::Utf8PathBuf;
use client::GoogleSheetsClient;
use config::{Config, SpreadsheetSpecification};
use console::Term;
use dir::{ParsedDataDir, confirm_move};
use itertools::Itertools;

const N_FIELDS: usize = 4;

pub async fn stage_data(config: &config::Config, data_dirs: &[Utf8PathBuf], skip_confirm: bool) -> anyhow::Result<()> {
    let Config {
        google_sheets_api_key,
        spreadsheet_spec,
        staging_dir_spec,
    } = config;

    let parsed_data_dirs: Vec<_> = data_dirs
        .iter()
        .map(ParsedDataDir::from_dir)
        .try_collect()
        .context("failed to parse Xenium data directory")?;

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
                .context("failed to construct new data directory names")
        })
        .try_collect()?;

    let mut move_futures = Vec::new();
    let term = Term::stdout();
    for renaming in &renamings {
        for (old_path, new_path) in renaming {
            let mut push_future = || move_futures.push(dir::rename(old_path, new_path));

            if skip_confirm {
                push_future();
            } else if confirm_move(&term, old_path, new_path)? {
                push_future();
                term.write_line("")?;
            }
        }
    }

    futures::future::try_join_all(move_futures).await?;

    Ok(())
}
