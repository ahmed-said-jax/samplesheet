use std::str::FromStr;

use camino::Utf8PathBuf;
use reqwest::{Method, Request, RequestBuilder, Url};
use serde::Deserialize;

use super::N_SPREADSHEET_RANGES;

#[derive(Deserialize)]
pub struct Config {
    pub(super) google_sheets_api_key: String,
    pub(super) spreadsheet: Spreadsheet,
    #[serde(default = "default_staging_dir")]
    pub(super) staging_dir: Utf8PathBuf,
}

#[derive(Deserialize)]
pub(super) struct Spreadsheet {
    pub id: String,
    sheet_name: String,
    lab_name_range: String,
    run_id_range: String,
    slide_id_range: String,
    slide_name_range: String,
}
impl Spreadsheet {
    pub fn to_a1_ranges(&self) -> [String; N_SPREADSHEET_RANGES] {
        let Self {
            sheet_name,
            lab_name_range,
            run_id_range,
            slide_id_range,
            slide_name_range,
            ..
        } = self;

        [lab_name_range, run_id_range, slide_id_range, slide_name_range].map(|r| format!("'{sheet_name}'!{r}"))
    }
}

#[derive(Deserialize)]
struct Assay {
    lab_name_range: String,
    run_id_range: String,
    slide_id_range: String,
}

#[derive(Deserialize)]
struct Slide {
    slide_id_range: String,
    slide_name_range: String,
}

fn default_staging_dir() -> Utf8PathBuf {
    Utf8PathBuf::from_str("/sc/service/staging").unwrap()
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::Spreadsheet;

    #[test]
    fn range_notation() {
        let spreadsheet_config = Spreadsheet {
            id: "amazing ID".to_string(),
            sheet_name: "awesome name".to_string(),
            lab_name_range: "A1:A10".to_string(),
            run_id_range: "B1:B10".to_string(),
            slide_id_range: "C1:C10".to_string(),
            slide_name_range: "D1:D10".to_string(),
        };

        assert_eq!(
            [
                "'awesome name'!A1:A10",
                "'awesome name'!B1:B10",
                "'awesome name'!C1:C10",
                "'awesome name'!D1:D10"
            ],
            spreadsheet_config.to_a1_ranges()
        );
    }
}
