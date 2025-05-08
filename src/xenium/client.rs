use std::str::FromStr;

use anyhow::{Context, ensure};
use itertools::Itertools;
use reqwest::{
    Client, ClientBuilder, Url,
    header::{HeaderMap, HeaderValue},
};

use super::{
    config::SpreadsheetSpecification,
    spreadsheet::{self, MajorDimension},
};

#[derive(Clone)]
pub(super) struct GoogleSheetsClient(Client);

impl GoogleSheetsClient {
    pub fn new(api_key: &str) -> anyhow::Result<Self> {
        let mut api_key = HeaderValue::from_str(api_key)?;
        api_key.set_sensitive(true);

        let mut headers = HeaderMap::new();

        // This is the dumbest header key I've ever seen
        headers.insert("X-goog-api-key", api_key);

        let client = ClientBuilder::new().default_headers(headers).build()?;

        Ok(Self(client))
    }

    pub async fn download_spreadsheet(
        &self,
        spec: &SpreadsheetSpecification,
    ) -> anyhow::Result<spreadsheet::ValueRange> {
        let Self(client) = self;

        let url = spec
            .to_url()
            .context("failed to construct URL from spreadsheet specification")?;
        let request = client
            .get(url)
            .query(&[("majorDimension", &MajorDimension::Rows.to_string())]);

        let raw_data = request.send().await?.text().await?;

        let data =
            serde_json::from_str(&raw_data).context(format!("failed to deserialize spreadsheet:\n{raw_data}"))?;

        Ok(data)
    }
}
