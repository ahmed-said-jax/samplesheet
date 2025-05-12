use std::{collections::HashMap, fmt::Display};

use itertools::Itertools;
use serde::{Deserialize, Deserializer};

use super::{N_FIELDS, SpreadsheetSpecification, slide::Slide};

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct ValueRange {
    range: String,
    major_dimension: MajorDimension,
    #[serde(deserialize_with = "vec_to_array")]
    values: Vec<[String; N_FIELDS]>,
}

impl ValueRange {
    pub(super) fn to_xenium_slides<'a>(
        &'a self,
        spec: &SpreadsheetSpecification,
    ) -> anyhow::Result<HashMap<&'a str, Vec<Slide<'a>>>> {
        let Self { values, .. } = self;

        let SpreadsheetSpecification {
            slide_id_col_idx,
            slide_name_col_idx,
            run_id_col_idx,
            lab_name_col_idx,
            ..
        } = spec;

        let slides = values
            .iter()
            .map(|array| {
                Slide::builder()
                    .id(&array[*slide_id_col_idx as usize])
                    .name(&array[*slide_name_col_idx as usize])
                    .run_id(&array[*run_id_col_idx as usize])
                    .lab_name(&array[*lab_name_col_idx as usize])
                    .build()
            })
            .into_group_map_by(|s| s.run_id);

        Ok(slides)
    }
}

fn vec_to_array<'de, D>(deserializer: D) -> Result<Vec<[String; N_FIELDS]>, D::Error>
where
    D: Deserializer<'de>,
{
    let vec_of_vecs = <Vec<Vec<String>>>::deserialize(deserializer)?;

    let to_array = |v: Vec<String>| {
        let err = serde::de::Error::custom(format!("expected array of length {N_FIELDS}, found {v:?}"));

        Ok(v.into_iter().collect_array().ok_or(err)?)
    };

    let vec_of_arrays = vec_of_vecs
        .into_iter()
        .filter(|v| v.len() != 0)
        .map(to_array)
        .try_collect()?;

    Ok(vec_of_arrays)
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "UPPERCASE")]
pub(super) enum MajorDimension {
    Rows,
}
impl Display for MajorDimension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "ROWS".fmt(f)
    }
}
