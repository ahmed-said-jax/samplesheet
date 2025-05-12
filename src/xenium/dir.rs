use std::collections::{HashMap, HashSet};

use anyhow::{Context, anyhow, ensure};
use camino::{Utf8Path, Utf8PathBuf};
use console::Term;

use super::{config::StagingDirSpecification, slide::Slide};

// The format for a xenium output file, somewhat stupidly, is:
// ├── <DATE>__<SOME STRING>__<RUN ID>
// │   ├── output-<MACHINE ID>__<SLIDE NAME>__<REGION NAME>__<DATE>__<SOME STRING>
//
// This is dumb because we're using the SLIDE NAME instead of the SLIDE ID. The latter is made by us and guaranteed to
// be unique.
//
// We want to rearrange this such that each of those subdirectories above is extracted and delivered as:
// └── <SLIDE ID>-<RUN ID>_<SLIDE NAME>
//     ├── design
//     └── xeniumranger

#[derive(Debug)]
pub(super) struct ParsedDataDir<'a> {
    path: &'a Utf8Path,
    run_id: &'a str,
    subdirs: Vec<SubDir>,
}

#[derive(Debug)]
struct SubDir {
    path: Utf8PathBuf,
    slide_name: String, // This is so stupid
}

impl<'a> ParsedDataDir<'a> {
    pub(super) fn from_dir(dir: &'a impl AsRef<Utf8Path>) -> anyhow::Result<Self> {
        let dir = dir.as_ref();
        let dir_name = dir
            .file_name()
            .ok_or(anyhow!("failed to decode top-level directory name"))?;

        let run_id = dir_name
            .split("__")
            .last()
            .ok_or(anyhow!("failed to get run ID in top-level Xenium data directory"))?;

        let raw_subdirs = dir
            .read_dir_utf8()
            .context(format!("failed to read sub-directories of {dir}"))?;
        const MAX_SUBDIRS: usize = 8;
        let mut subdirs = Vec::with_capacity(MAX_SUBDIRS);
        for subdir in raw_subdirs {
            let subdir = subdir?;

            let path = subdir.into_path();

            let path_name = path.file_name().ok_or(anyhow!("failed to get filename for {path}"))?;

            if path_name == ".gitkeep" {
                continue;
            }

            let slide_name = path_name
                .split("__")
                .nth(1)
                .ok_or(anyhow!("failed to get slide name for {path}"))?
                .to_string();

            subdirs.push(SubDir { path, slide_name });
        }

        Ok(Self {
            path: dir,
            run_id,
            subdirs,
        })
    }

    // Should this function be split into one that finds the matching slides and one that renames?
    pub(super) fn construct_new_subdir_names<'b>(
        &'a self,
        slides: &'b HashMap<&str, Vec<Slide>>,
        staging_dir_spec: &StagingDirSpecification,
    ) -> anyhow::Result<Vec<(&'b Utf8Path, Utf8PathBuf)>>
    where
        'a: 'b,
    {
        let Self { path, run_id, subdirs } = self;
        let matching_slides = slides.get(run_id).ok_or(anyhow!(
            "failed to find Xenium slides from spreadsheet from path {path}"
        ))?;

        ensure!(
            matching_slides.len() == subdirs.len(),
            "spreadsheet indicates {} slides for this run, but the data directory {path} has {} subdirectories (each \
             corresponding to a slide)",
            matching_slides.len(),
            subdirs.len()
        );

        let mut subdirs_paired_with_slides = Vec::new();
        let mut seen_spreadsheet_slide_names = HashSet::new();
        for SubDir {
            path,
            slide_name: subdir_slide_name,
        } in subdirs
        {
            for Slide {
                id: spreadsheet_slide_id,
                name: spreadsheet_slide_name,
                run_id,
                lab_name,
            } in matching_slides
            {
                if subdir_slide_name != spreadsheet_slide_name {
                    continue;
                }

                let is_unseen = seen_spreadsheet_slide_names.insert(*spreadsheet_slide_name);
                ensure!(
                    is_unseen,
                    "found multiple slides with the name {spreadsheet_slide_name} in the spreadsheet"
                );

                let new_path_name = format!("{spreadsheet_slide_id}-{run_id}_{spreadsheet_slide_name}");

                let lab_staging_dir = staging_dir_spec.lab_staging_dir(*lab_name)?;
                let lab_staging_dir = lab_staging_dir
                    .canonicalize_utf8()
                    .context(format!("failed to get absolute path for {lab_staging_dir}"))?;

                let new_path = lab_staging_dir.join(new_path_name).join("xeniumranger");

                subdirs_paired_with_slides.push((path.as_path(), new_path));
            }
        }

        Ok(subdirs_paired_with_slides)
    }
}

pub(super) fn confirm_move(term: &Term, old_path: &Utf8Path, new_path: &Utf8Path) -> anyhow::Result<bool> {
    let err = "failed to write line to terminal";
    if new_path.exists() {
        term.write_line(&format!(
            "skipping renaming {old_path} to {new_path}, as it already exists"
        ))
        .context(err)?;
    }

    term.write_line(&format!("move {old_path} -> {new_path} (y/n)?"))
        .context(err)?;
    let res = loop {
        let input = term.read_char().context("failed to read char from terminal")?;
        if input == 'y' {
            break true;
        } else if input == 'n' {
            break false;
        } else {
            term.write_line("input must be either 'y' or 'n'").context(err)?;
        }
    };

    Ok(res)
}

pub(super) async fn rename(old_path: &Utf8Path, new_path: &Utf8Path) -> anyhow::Result<()> {
    let old_path = old_path
        .canonicalize_utf8()
        .context(format!("failed to get absolute path for {old_path}"))?;

    if new_path.exists() {
        println!("skipping renaming {old_path} to {new_path}, as it already exists");
    }

    tokio::fs::create_dir_all(new_path.join("design"))
        .await
        .context("failed to create directory {new_path}")?;

    let mut mv_cmd = tokio::process::Command::new("mv");
    mv_cmd.arg(&old_path).arg(&new_path);

    let output = mv_cmd
        .output()
        .await
        .context(format!("failed to move {old_path} to {new_path}"))?;

    ensure!(
        output.status.success(),
        "failed to move {old_path} to {new_path}:\n{}",
        String::from_utf8(output.stderr).unwrap()
    );

    Ok(())
}
