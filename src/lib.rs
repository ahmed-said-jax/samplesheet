use std::{collections::HashMap, fs, hash::{Hash, RandomState}};
use anyhow::{anyhow, Context};
use itertools::Itertools;
use serde::Deserialize;
use camino::{Utf8Path, Utf8PathBuf};
use tracking_sheet::{FromTrackingSheetDir, Gems, GemsSuspensions, Id, Library, MultiplexedSuspension, Suspension, TrackingSheet};
mod tracking_sheet;

pub fn generate_samplesheet(config_path: &Utf8Path, fastq_paths: &[Utf8PathBuf], tracking_sheet_dir: &Utf8Path) -> anyhow::Result<Samplesheet> {
    let config = Config::from_path(config_path).context(format!("failed to read configuration file at {config_path}"))?;

    let fastq_paths = library_id_to_fastq_dir(fastq_paths).context("failed to determine library IDs from FASTQ paths")?;

    fn from_tracking_sheet_dir<T: FromTrackingSheetDir>(dir: &Utf8Path) -> anyhow::Result<Vec<T>> {
        T::from_tracking_sheet_dir(dir)
    }

    let suspensions = Suspension::from_tracking_sheet_dir(tracking_sheet_dir)?;
    let suspensions = map_entity_id_to_entity(&suspensions);

    let multiplexed_suspensions = MultiplexedSuspension::from_tracking_sheet_dir(tracking_sheet_dir)?;
    let multiplexed_suspensions = map_entity_id_to_entity(&multiplexed_suspensions);

    let gems = Gems::from_tracking_sheet_dir(tracking_sheet_dir)?;
    let gems_suspensions = GemsSuspensions::from_tracking_sheet_dir(tracking_sheet_dir)?;

    let libraries = Library::from_tracking_sheet_dir(tracking_sheet_dir)?;
    let libraries = libraries.iter().filter_map(|l| fastq_paths.get(l.id.as_str()).map(|p| (l, *p))).filter(|(l, _)| l.fails_qc).sorted_by_key(|(l, _)| &l.gems_id).chunk_by(|(l, _)| &l.gems_id);

    let grouped_suspensions = gems_suspensions.iter().filter_map(|gs| gs.suspension_id.as_ref().map(|s| (&gs.gems_id, s)));
    let grouped_suspensions: HashMap<_, _, RandomState> = HashMap::from_iter(grouped_suspensions);

    let grouped_multiplexed_suspensions = gems_suspensions.iter().filter_map(|gs| gs.multiplexed_suspension_id.as_ref().map(|m| (&gs.gems_id, m)));
    let grouped_multiplexed_suspensions: HashMap<_, _, RandomState> = HashMap::from_iter(grouped_multiplexed_suspensions);

    let mut samplesheets = Vec::new();

    for (gems_id, libs) in &libraries {
        let suspension_id = grouped_suspensions.get(gems_id);
        let multiplexed_suspension_id = grouped_multiplexed_suspensions.get(gems_id);

        let sample_name = match (suspension_id, multiplexed_suspension_id) {
            (Some(id), None) => &suspensions.get(id.as_str()).ok_or(anyhow!(format!("suspension ID {id} not found")))?.name,
            (None, Some(id)) => &multiplexed_suspensions.get(id.as_str()).ok_or(anyhow!(format!("multiplexed suspension ID {id} not found")))?.name,
            (Some(_), Some(_)) => return Err(anyhow!("GEMs IDs {gems_id} is associated with both a suspension and a multiplexed suspension")),
            (None, None) => return Err(anyhow!("GEMs ID {gems_id} is associated with neither a suspension nor a multiplexed suspension"))
        };

        let samplesheet = Samplesheet{sample_name: sample_name.clone(), libraries: libs.map(|(l, p)| l.id).collect(), library_types};
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

fn map_entity_id_to_entity<T: Id>(entities: &[T]) -> HashMap<&str, &T> {
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
    design: Option<HashMap<String, SampleDesign>>,
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