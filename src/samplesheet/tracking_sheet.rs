use std::fs;

use anyhow::Context;
use camino::Utf8Path;
use itertools::Itertools;
use serde::{Deserialize, de::DeserializeOwned};

pub trait FromTrackingSheetDir: Sized + DeserializeOwned {
    fn filename() -> &'static str;

    fn from_tracking_sheet_dir(dir: &Utf8Path) -> anyhow::Result<Vec<Self>> {
        let path = dir.join(Self::filename());
        let contents = fs::read_to_string(&path)
            .context(format!("failed to read {path}"))?
            .split('\n')
            .skip(Self::header_row())
            .join("\n");

        let mut reader = csv::Reader::from_reader(contents.as_bytes());
        let records: anyhow::Result<Vec<_>, _> = reader.deserialize().collect();

        Ok(records?)
    }

    fn header_row() -> usize {
        0
    }
}

pub trait Id {
    fn id(&self) -> &str;
}

#[derive(Deserialize)]
pub struct Suspension {
    #[serde(rename = "Suspension ID")]
    pub id: String,
    #[serde(rename = "Specimen Name")]
    pub name: String,
    #[serde(rename = "Species")]
    pub species: String,
    #[serde(rename = "Cellular Material")]
    pub cellular_material: String,
    #[serde(rename = "Tissue")]
    pub tissue: String,
    #[serde(rename = "Multiplexing Tag ID")]
    pub tag_id: Option<String>,
    #[serde(rename = "Pooled Into ID")]
    pub pooled_into_id: Option<String>,
}
impl FromTrackingSheetDir for Suspension {
    fn filename() -> &'static str {
        "Chromium(Suspensions).csv"
    }

    fn header_row() -> usize {
        1
    }
}
impl Id for Suspension {
    fn id(&self) -> &str {
        &self.id
    }
}

#[derive(Deserialize)]
pub struct MultiplexedSuspension {
    #[serde(rename = "Multiplexed Suspension (Pool) ID")]
    id: String,
    #[serde(rename = "Multiplexed Suspension (Pool) Name")]
    pub name: String,
}
impl FromTrackingSheetDir for MultiplexedSuspension {
    fn filename() -> &'static str {
        "Chromium(Multiplexed Suspensions).csv"
    }

    fn header_row() -> usize {
        1
    }
}
impl Id for MultiplexedSuspension {
    fn id(&self) -> &str {
        &self.id
    }
}

#[derive(Deserialize)]
pub struct Gems {
    #[serde(rename = "GEMs ID")]
    id: String,
    #[serde(rename = "Chemistry")]
    pub chemistry: String,
}
impl FromTrackingSheetDir for Gems {
    fn filename() -> &'static str {
        "Chromium(GEMs).csv"
    }
}
impl Id for Gems {
    fn id(&self) -> &str {
        &self.id
    }
}

#[derive(Deserialize)]
pub struct GemsSuspensions {
    #[serde(rename = "GEMs ID")]
    pub gems_id: String,
    #[serde(rename = "Suspension ID")]
    pub suspension_id: Option<String>,
    #[serde(rename = "Multiplexed Suspension ID")]
    pub multiplexed_suspension_id: Option<String>,
}
impl FromTrackingSheetDir for GemsSuspensions {
    fn filename() -> &'static str {
        "Chromium(GEMs-Suspensions).csv"
    }
}

#[derive(Deserialize)]
pub struct Library {
    #[serde(rename = "Library ID")]
    pub id: String,
    #[serde(rename = "GEMs ID")]
    pub gems_id: String,
    #[serde(rename = "Library Type")]
    pub type_: String,
}
impl FromTrackingSheetDir for Library {
    fn filename() -> &'static str {
        "Chromium(Libraries).csv"
    }
}
impl Id for Library {
    fn id(&self) -> &str {
        &self.id
    }
}
