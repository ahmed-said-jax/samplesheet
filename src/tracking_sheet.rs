use std::collections::HashMap;

use camino::Utf8Path;
use serde::{de::DeserializeOwned, Deserialize};

pub struct TrackingSheet {
    pub suspensions: Vec<Suspension>,
    pub multiplexed_suspensions: Vec<MultiplexedSuspension>,
    pub gems: Vec<Gems>,
    pub gems_suspensions: Vec<GemsSuspensions>,
    pub libraries: Vec<Library>,
}

pub trait FromTrackingSheetDir: Sized + DeserializeOwned {
    fn filename() -> &'static str;

    fn from_tracking_sheet_dir(dir: &Utf8Path) -> anyhow::Result<Vec<Self>> {
        let path = dir.join(Self::filename());

        let mut reader = csv::Reader::from_path(path)?;
        let records: anyhow::Result<Vec<_>, _> = reader.deserialize().collect();

        Ok(records?)
    }
}

pub trait Id {
    fn id(&self) -> &str;
}

#[derive(Deserialize)]
pub struct Suspension {
    #[serde(rename = "Suspension ID")]
    id: String,
    #[serde(rename = "Specimen Name")]
    pub name: String,
    #[serde(rename = "Species")]
    species: String,
    #[serde(rename = "Cellular Material")]
    cellular_material: String
}
impl FromTrackingSheetDir for Suspension {
    fn filename() -> &'static str {
        "Chromium(Suspensions).csv"
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
    chemistry: String,

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
    pub multiplexed_suspension_id: Option<String>
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
    #[serde(rename = "Fails QC")]
    pub fails_qc: bool

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
