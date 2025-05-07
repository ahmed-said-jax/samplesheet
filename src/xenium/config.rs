use std::{fmt::Display, str::FromStr};

use anyhow::{anyhow, ensure};
use camino::Utf8PathBuf;
use itertools::Itertools;
use regex::Regex;
use serde::Deserialize;

use super::N_SPREADSHEET_RANGES;

#[derive(Deserialize)]
pub struct Config {
    pub(super) google_sheets_api_key: String,
    pub(super) spreadsheet_spec: SpreadsheetSpecification,
    #[serde(default = "default_staging_dir")]
    pub(super) staging_dir: Utf8PathBuf,
}

#[derive(Deserialize)]
pub(super) struct SpreadsheetSpecification {
    pub id: String,
    sheet_name: String,
    lab_name_range: SpreadsheetColumnRange,
    run_id_range: SpreadsheetColumnRange,
    slide_id_range: SpreadsheetColumnRange,
    slide_name_range: SpreadsheetColumnRange,
}
impl SpreadsheetSpecification {
    pub fn validate_n_rows(&self) -> anyhow::Result<()> {
        let Self {
            lab_name_range: SpreadsheetColumnRange((_, lab_row1), (_, lab_row2)),
            run_id_range: SpreadsheetColumnRange((_, run_row1), (_, run_row2)),
            slide_id_range: SpreadsheetColumnRange((_, slide_id_row1), (_, slide_id_row2)),
            slide_name_range: SpreadsheetColumnRange((_, slide_name_row1), (_, slide_name_row2)),
            ..
        } = self;

        ensure!(
            [lab_row1, run_row1, slide_id_row1, slide_name_row1].iter().all_equal(),
            "the starting row of each range must be equal"
        );

        ensure!(
            [lab_row2, run_row2, slide_id_row2, slide_name_row2].iter().all_equal(),
            "the end row of each range must be equal"
        );

        Ok(())
    }

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

#[derive(PartialEq, Debug)]
pub struct SpreadsheetColumnRange((String, String), (String, String));
impl FromStr for SpreadsheetColumnRange {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let cell_regex = r#"([A-Z]+)([1-9][0-9]?)"#;
        let range_regex = Regex::new(&format!(r#"^{cell_regex}:{cell_regex}$"#)).unwrap();

        let captures = range_regex
            .captures(s)
            .ok_or(anyhow!("range must match {range_regex}"))?;

        let [_, col1, row1, col2, row2] = captures
            .iter()
            .next_array()
            .unwrap()
            .map(|m| m.unwrap().as_str().to_string());

        ensure!(col1 == col2, "range must correspond to exactly one column");

        Ok(Self((col1, row1), (col2, row2)))
    }
}

impl Display for SpreadsheetColumnRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self((s1, s2), (s3, s4)) = self;

        format!("{s1}{s2}:{s3}{s4}").fmt(f)
    }
}

impl<'de> Deserialize<'de> for SpreadsheetColumnRange {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw_range = String::deserialize(deserializer)?;

        Ok(Self::from_str(&raw_range).map_err(|e| serde::de::Error::custom(e.to_string()))?)
    }
}

fn default_staging_dir() -> Utf8PathBuf {
    Utf8PathBuf::from_str("/sc/service/staging").unwrap()
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use pretty_assertions::{assert_eq, assert_str_eq};
    use serde::Deserialize;
    

    use super::SpreadsheetColumnRange;

    #[test]
    fn spreadsheet_range_to_and_from_str() {
        let raw_range = "A1:A10";
        let from_str = SpreadsheetColumnRange::from_str(raw_range).unwrap();

        let [col1, row1, col2, row2] = ["A", "1", "A", "10"].map(|s| s.to_string());
        let expected_range = SpreadsheetColumnRange((col1, row1), (col2, row2));

        assert_eq!(from_str, expected_range);

        assert_str_eq!(raw_range, from_str.to_string())
    }

    #[test]
    fn excess_columns_in_range() {
        let raw_range = "A1:B10";

        SpreadsheetColumnRange::from_str(raw_range).unwrap_err();
    }

    #[test]
    fn deserialize_spreadsheet_range() {
        #[derive(Deserialize)]
        struct TestStruct {
            _range: SpreadsheetColumnRange,
        }

        let toml = r#"_range = "A1:A10""#;
        let _: TestStruct = toml::from_str(toml).unwrap();
    }
}
