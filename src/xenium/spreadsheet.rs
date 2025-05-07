use std::mem;

use anyhow::{Context, ensure};
use itertools::Itertools;
use serde::{Deserialize, Deserializer};

use super::{
    N_SPREADSHEET_RANGES,
    slide::{Slide},
};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct Spreadsheet {
    pub(super) spreadsheet_id: String,
    value_ranges: [ValueRange; N_SPREADSHEET_RANGES],
}

impl Spreadsheet {
    pub(super) fn to_xenium_slides<'a>(&'a self) -> anyhow::Result<Vec<Slide<'a>>> {
        let Self { value_ranges, .. } = self;
        let as_rows = value_ranges
            .transpose()
            .context("failed to transpose spreadsheet into array of rows")?;

        let slides = as_rows
            .iter()
            .map(|[lab_name, run_id, slide_id, slide_name]| {
                Slide::builder()
                    .lab_name(lab_name)
                    .run_id(run_id)
                    .id(slide_id)
                    .name(slide_name)
                    .build()
            })
            .collect();

        Ok(slides)
    }
}

trait AsArray<'a> {
    fn as_array(&self) -> anyhow::Result<[&'a str; N_SPREADSHEET_RANGES]>;
}

impl<'a> AsArray<'a> for (((&'a String, &'a String), &'a String), &'a String) {
    fn as_array(&self) -> anyhow::Result<[&'a str; N_SPREADSHEET_RANGES]> {
        let (((s0, s1), s2), s3) = self;

        let array = [s0, s1, s2, s3].map(|s| s.as_str());

        ensure!(
            array.iter().map(|s| s.is_empty()).all_equal(),
            "all values in row must be either empty or full, but the following row is neither:\n{array:?}"
        );

        Ok(array)
    }
}

trait Transpose {
    fn transpose(&self) -> anyhow::Result<Vec<[&str; 4]>>;
}

impl Transpose for [ValueRange; N_SPREADSHEET_RANGES] {
    fn transpose(&self) -> anyhow::Result<Vec<[&str; 4]>> {
        let [lab_name_col, run_id_col, slide_id_col, slide_name_col] = self.each_ref().map(|vr| &vr.values);

        let as_array = lab_name_col
            .iter()
            .zip(run_id_col)
            .zip(slide_id_col)
            .zip(slide_name_col)
            .map(|strings| strings.as_array())
            .try_collect()?;

        Ok(as_array)
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ValueRange {
    range: String,
    major_dimension: MajorDimension,
    #[serde(deserialize_with = "flatten_array")]
    values: Vec<String>,
}

fn flatten_array<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let mut nested = <[Vec<String>; 1]>::deserialize(deserializer)?;

    // I love Rust
    Ok(mem::take(&mut nested[0]))
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "UPPERCASE")]
enum MajorDimension {
    Columns,
}
