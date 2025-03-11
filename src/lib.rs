use std::{collections::HashMap, fs, hash::{Hash, RandomState}};
use anyhow::{anyhow, Context};
use itertools::Itertools;
use serde::Deserialize;
use camino::{Utf8Path, Utf8PathBuf};
use tracking_sheet::{FromTrackingSheetDir, Id, Suspension, TrackingSheet};
mod tracking_sheet;

pub fn generate_samplesheet(config_path: &Utf8Path, fastq_paths: &[Utf8PathBuf], tracking_sheet_dir: &Utf8Path) -> anyhow::Result<Samplesheet> {
    let config = Config::from_path(config_path).context(format!("failed to read configuration file at {config_path}"))?;

    let fastq_paths = library_id_to_fastq_dir(fastq_paths).context("failed to determine library IDs from FASTQ paths")?;

    fn from_tracking_sheet_dir<T: FromTrackingSheetDir>(dir: &Utf8Path) -> anyhow::Result<Vec<T>> {
        T::from_tracking_sheet_dir(dir)
    }

    let tracking_sheet = TrackingSheet {suspensions: from_tracking_sheet_dir(tracking_sheet_dir)?, multiplexed_suspensions: from_tracking_sheet_dir(tracking_sheet_dir)?, gems: from_tracking_sheet_dir(tracking_sheet_dir)?, gems_suspensions: from_tracking_sheet_dir(tracking_sheet_dir)?, libraries: from_tracking_sheet_dir(tracking_sheet_dir)?};

    let libraries = tracking_sheet.libraries.iter().filter_map(|l| fastq_paths.get(l.id.as_str()).map(|p| (l, *p))).filter(|(l, _)| l.fails_qc).sorted_by_key(|(l, _)| &l.gems_id).chunk_by(|(l, _)| &l.gems_id);

    let suspensions = tracking_sheet.gems_suspensions.iter().filter_map(|gs| gs.suspension_id.as_ref().map(|s| (&gs.gems_id, s)));
    let suspensions: HashMap<_, _, RandomState> = HashMap::from_iter(suspensions);

    let multiplexed_suspensions = tracking_sheet.gems_suspensions.iter().filter_map(|gs| gs.multiplexed_suspension_id.as_ref().map(|m| (&gs.gems_id, m)));
    let multiplexed_suspensions: HashMap<_, _, RandomState> = HashMap::from_iter(multiplexed_suspensions);

    for (gems_id, libs) in &libraries {
        let suspension = suspensions.get(gems_id);
        let multiplexed_suspension = multiplexed_suspensions.get(gems_id);

        let Some()
    }

    todo!()
}

fn library_id_to_fastq_dir(fastq_paths: &[Utf8PathBuf]) -> anyhow::Result<HashMap<&str, &Utf8Path>>{
    let mut library_ids_to_fastqs = HashMap::new();
    for p in fastq_paths {
        let err = format!("malformed FASTQ path: {p}");

        let filename = p.file_name().ok_or_else(|| anyhow!(err.clone()))?;
        let dir = p.parent().ok_or_else(|| anyhow!(err))?;

        library_ids_to_fastqs.insert(filename, dir);
    }

    Ok(library_ids_to_fastqs)
}

fn entity_id_to_entity<T: Id>(entities: &[T]) -> HashMap<&str, &T> {
    let map = entities.iter().map(|e| (e.id(), e));

    HashMap::from_iter(map)
}

pub struct Samplesheet {
    libraries: Vec<String>,
    sample_name: String,
    library_types: Vec<String>,
    tool: String,
    tool_version: String,
    command: String,
    reference_path: Utf8PathBuf,
    probe_set: Utf8PathBuf,
    design: HashMap<String, SampleDesign>
    fastq_paths: Vec<String>,
}

struct SampleDesign {
    name: String,
    description: String
}

#[derive(Deserialize)]
struct Config {
    reference_paths: HashMap<String, String>,
    chemistry_tool: HashMap<String, String>,
    nf_tenx_repo: String,
    probe_set_paths: HashMap<String, String>,
}

impl Config {
    fn from_path(path: &Utf8Path) -> anyhow::Result<Self> {
        let contents = fs::read_to_string(&path)?;
        Ok(toml::from_str(&contents)?)
    }
}