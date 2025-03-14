use std::{collections::HashMap, fs};

use anyhow::{Context, anyhow};
use camino::{Utf8Path, Utf8PathBuf};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tracking_sheet::{FromTrackingSheetDir, Gems, GemsSuspensions, Id, Library, MultiplexedSuspension, Suspension};
mod tracking_sheet;

pub fn write_samplesheet(
    config_path: &Utf8Path,
    fastq_paths: &[Utf8PathBuf],
    tracking_sheet_dir: &Utf8Path,
    output_path: &Utf8Path,
) -> anyhow::Result<()> {
    let config =
        Config::from_path(config_path).context(format!("failed to read configuration file at {config_path}"))?;

    let fastq_paths =
        library_id_to_fastq_dir(fastq_paths).context("failed to determine library IDs from FASTQ paths")?;

    let suspensions = Suspension::from_tracking_sheet_dir(tracking_sheet_dir)?;
    let suspensions_grouped_by_pool = suspensions
        .iter()
        .filter_map(|s| s.pooled_into_id.as_ref().map(|m| (m.as_str(), s)))
        .into_group_map();
    let suspensions = map_entity_id_to_entity(&suspensions);

    let multiplexed_suspensions = MultiplexedSuspension::from_tracking_sheet_dir(tracking_sheet_dir)?;
    let multiplexed_suspensions = map_entity_id_to_entity(&multiplexed_suspensions);

    let gems = Gems::from_tracking_sheet_dir(tracking_sheet_dir)?;
    let gems = map_entity_id_to_entity(&gems);

    let gems_suspensions = GemsSuspensions::from_tracking_sheet_dir(tracking_sheet_dir)?;

    let suspension_ids_grouped_by_gems_id = gems_suspensions
        .iter()
        .filter_map(|gs| gs.suspension_id.as_ref().map(|s| (gs.gems_id.as_str(), s.as_str())))
        .into_group_map();

    let multiplexed_suspension_ids_grouped_by_gems_id = gems_suspensions
        .iter()
        .filter_map(|gs| {
            gs.multiplexed_suspension_id
                .as_ref()
                .map(|s| (gs.gems_id.as_str(), s.as_str()))
        })
        .into_group_map();

    let libraries = Library::from_tracking_sheet_dir(tracking_sheet_dir)?;
    let libraries_grouped_by_gems_id = libraries
        .iter()
        .filter_map(|l| fastq_paths.get(l.id()).map(|p| (l.gems_id.as_str(), (l, *p))))
        .into_group_map();

    let mut samplesheets = Vec::new();

    for (gems_id, libs) in &libraries_grouped_by_gems_id {
        let mut library_ids = Vec::new();
        let mut library_types = Vec::new();
        let mut library_fastqs = Vec::new();

        for (Library { id, type_, .. }, fastq_dir) in libs {
            library_ids.push(id.as_str());
            library_fastqs.push(*fastq_dir);

            let type_ = match type_.as_str() {
                "Gene Expression Flex" => "Gene Expression",
                _ => type_,
            };

            library_types.push(type_);
        }

        let library_gems = gems.get(gems_id).ok_or(anyhow!("GEMs ID {gems_id} not found"))?;
        let (tool, tool_version, command) = config
            .chemistry_program
            .get(&library_gems.chemistry)
            .ok_or(anyhow!("chemistry {} not found in config", library_gems.chemistry))?;

        let sample = Sample::from_entities(
            gems_id,
            &suspension_ids_grouped_by_gems_id,
            &multiplexed_suspension_ids_grouped_by_gems_id,
            &suspensions,
            &suspensions_grouped_by_pool,
            &multiplexed_suspensions,
        )?;

        let design = sample.design();
        let design = match design {
            None => None,
            Some(d) => Some(d?.clone()),
        };

        let species = sample.species();

        let reference_paths = config.species_reference_path.get(species).ok_or(anyhow!(
            "species {species} not found in config's 'species_reference_path'"
        ))?;
        let reference_path = reference_paths.get(&format!("{tool} {command}")).ok_or(anyhow!(
            "chemistry {} not found in reference paths for {species}",
            library_gems.chemistry
        ))?;

        let probe_set = config.species_probe_set.get(species).ok_or(anyhow!(
            "species {species} not found in config's 'species_reference_probe_set'"
        ))?;

        let is_nuclei = sample.is_nuclei()?;

        let samplesheet = Samplesheet {
            sample_name: sample.name(),
            libraries: library_ids,
            library_types,
            is_nuclei,
            fastq_paths: library_fastqs,
            design,
            tool,
            tool_version,
            command,
            reference_path,
            probe_set,
        };

        samplesheets.push(samplesheet);
    }

    fs::write(output_path, serde_json::to_string_pretty(&samplesheets)?)
        .context(format!("failed to write samplesheet to {output_path}"))?;

    Ok(())
}

fn library_id_to_fastq_dir(fastq_paths: &[Utf8PathBuf]) -> anyhow::Result<HashMap<&str, &Utf8Path>> {
    let mut library_ids_to_fastqs = HashMap::new();
    for p in fastq_paths {
        let err = format!("malformed FASTQ path: {p}");

        let filename = p
            .file_name()
            .ok_or_else(|| anyhow!(err.clone()))?
            .split('_')
            .next()
            .unwrap_or_default();
        let dir = p.parent().ok_or_else(|| anyhow!(err))?;

        library_ids_to_fastqs.insert(filename, dir);
    }

    Ok(library_ids_to_fastqs)
}

fn map_entity_id_to_entity<T: Id>(entities: &[T]) -> HashMap<&str, &T> {
    let map = entities.iter().map(|e| (e.id(), e));

    HashMap::from_iter(map)
}

enum Sample<'a> {
    Singleplexed(&'a Suspension),
    Multiplexed(&'a MultiplexedSuspension, &'a [&'a Suspension]),
    Ocm(Vec<&'a Suspension>),
}

impl<'a> Sample<'a> {
    fn from_entities(
        gems_id: &str,
        suspension_ids_grouped_by_gems_id: &HashMap<&str, Vec<&str>>,
        multiplexed_suspension_ids_grouped_by_gems_id: &'a HashMap<&str, Vec<&str>>,
        suspensions: &HashMap<&str, &'a Suspension>,
        suspensions_grouped_by_pool: &'a HashMap<&str, Vec<&'a Suspension>>,
        multiplexed_suspensions: &'a HashMap<&str, &'a MultiplexedSuspension>,
    ) -> anyhow::Result<Sample<'a>> {
        let suspension_id = suspension_ids_grouped_by_gems_id.get(gems_id);
        let multiplexed_suspension_id = multiplexed_suspension_ids_grouped_by_gems_id.get(gems_id);

        match (suspension_id, multiplexed_suspension_id) {
            (Some(ids), None) => {
                let sample = if ids.len() == 1 {
                    Sample::Singleplexed(
                        suspensions
                            .get(ids[0])
                            .ok_or(anyhow!("suspension ID {} not found", ids[0]))?,
                    )
                } else {
                    Sample::Ocm(
                        ids.iter()
                            .map(|id| {
                                suspensions
                                    .get(id)
                                    .map(|s| *s)
                                    .ok_or(anyhow!("suspension ID {id} not found"))
                            })
                            .try_collect()?,
                    )
                };

                Ok(sample)
            }
            (None, Some(ids)) => {
                let err = format!("multiplexed suspension ID {} not found", ids[0]);

                Ok(Sample::Multiplexed(
                    multiplexed_suspensions.get(ids[0]).ok_or(anyhow!(err.clone()))?,
                    &suspensions_grouped_by_pool.get(ids[0]).ok_or(anyhow!(err))?,
                ))
            }
            (Some(_), Some(_)) => Err(anyhow!(
                "GEMs IDs {gems_id} is associated with both a suspension and a multiplexed suspension"
            )),
            (None, None) => Err(anyhow!(
                "GEMs ID {gems_id} is associated with neither a suspension nor a multiplexed suspension"
            )),
        }
    }

    fn name(&self) -> &'a str {
        match self {
            Self::Singleplexed(Suspension { name, .. }) => name,
            Self::Multiplexed(MultiplexedSuspension { name, .. }, _) => name,
            Self::Ocm(_) => "OCM-pool",
        }
    }

    fn design(&self) -> Option<anyhow::Result<HashMap<&'a str, SampleDesign<'a>>>> {
        let suspensions = match self {
            Self::Singleplexed(_) => {
                return None;
            }
            Self::Multiplexed(_, suspensions) => *suspensions,
            Self::Ocm(suspensions) => suspensions.as_slice(),
        };

        let mut design = HashMap::with_capacity(suspensions.len());
        for Suspension {
            id,
            tag_id,
            name,
            tissue,
            ..
        } in suspensions
        {
            let Some(tag_id) = tag_id else {
                return Some(Err(anyhow!("no tag ID for suspension {id}")));
            };

            design.insert(
                tag_id.as_str(),
                SampleDesign {
                    name,
                    description: tissue,
                },
            );
        }

        Some(Ok(design))
    }

    fn species(&self) -> &'a str {
        match self {
            Self::Singleplexed(Suspension { species, .. }) => species,
            Self::Multiplexed(_, suspensions) => &suspensions[0].species,
            Self::Ocm(suspensions) => &suspensions[0].species,
        }
    }

    fn is_nuclei(&self) -> anyhow::Result<bool> {
        let cellular_material = match self {
            Self::Singleplexed(Suspension { cellular_material, .. }) => cellular_material.as_str(),
            Self::Multiplexed(_, suspensions) => &suspensions[0].cellular_material,
            Self::Ocm(suspensions) => &suspensions[0].cellular_material,
        };

        let is_nuclei = match cellular_material {
            "Cells" => false,
            "Nuclei" => true,
            _ => return Err(anyhow!("unrecognized cellular material {cellular_material}")),
        };

        Ok(is_nuclei)
    }
}

#[derive(Serialize)]
pub struct Samplesheet<'a> {
    libraries: Vec<&'a str>,
    sample_name: &'a str,
    library_types: Vec<&'a str>,
    is_nuclei: bool,
    tool: &'a str,
    tool_version: &'a str,
    command: &'a str,
    reference_path: &'a Utf8Path,
    probe_set: &'a Utf8Path,
    design: Option<HashMap<&'a str, SampleDesign<'a>>>,
    fastq_paths: Vec<&'a Utf8Path>,
}

#[derive(Serialize, Clone)]
struct SampleDesign<'a> {
    name: &'a str,
    description: &'a str,
}

#[derive(Deserialize)]
struct Config {
    species_reference_path: HashMap<String, HashMap<String, Utf8PathBuf>>,
    chemistry_program: HashMap<String, (String, String, String)>,
    species_probe_set: HashMap<String, Utf8PathBuf>,
}

impl Config {
    fn from_path(path: &Utf8Path) -> anyhow::Result<Self> {
        let contents = fs::read_to_string(&path)?;
        Ok(toml::from_str(&contents)?)
    }
}
