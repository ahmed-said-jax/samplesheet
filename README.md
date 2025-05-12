# scbl-utils
A command-line utility for data-processing and delivery at the Single Cell Biology Laboratory at the Jackson Laboratory

## Installation
Install and update with [cargo](https://github.com/rust-lang/cargo):
```bash
cargo install scbl-utils
```
## Usage
**Note**: All of the below information is available by invoking `scbl-utils --help`.
### Configuration
`scbl-utils` requires a configuration file. By default, it looks for this file at `/sc/service/.config/scbl-utils/config.toml`, but you can override that by setting the environment variable `SCBL_UTILS_CONFIG_PATH` or on the command-line by doing:
```bash
scbl-utils --config-path /path/to/config.toml <COMMAND>
```
See [config.sample.toml](config.sample.toml) for a nearly complete example that should "just work" on `elion`, provided you fill the fields `xenium.google_sheets_api_key` and `xenium.spreadsheet_spec.id`.
### Cache
Similarly, `scbl-utils` utilizes a cache directory to prevent downloading recently-fetched resources. By default, this cache directory is `/sc/service/.cache/scbl-utils/`, but you can alter that with the environment variable `SCBL_UTILS_CACHE_DIR` or on the command-line:
```bash
scbl-utils --cache-dir /path/to/cache <COMMAND>
```
### Generate an [`nf-tenx`](https://github.com/thejacksonlaboratory/nf-tenx) Samplesheet
1. Download the 5 spreadsheets that make up the Chromium workbook as CSV files.
2. Put them in one directory. By default, `scbl-utils` will look for the CSV files at `/sc/service/.cache/scbl-utils/chromium-tracking-sheet`, but you can [override this behavior](#cache). However, note that overriding this behavior may lead to errors, as other users may find outdated tracking sheets at `/sc/service/.cache/scbl-utils/chromium-tracking-sheet`, or they may end up duplicating your work without knowledge of where you put the tracking sheet.
3. Name each file with the name of its Excel sheet. Currently, these are:
    - `Suspensions.csv`
    - `Multiplexed Suspensions.csv`
    - `GEMs.csv`
    - `GEMs-Suspensions.csv`
    - `Libraries.csv`
4. Run the script, passing in a list of `fastq` **files**. **Do not** pass in a list of directories - this will throw an error (by design). For most use cases, you can use globs on GT delivery directories:
```bash
scbl-utils samplesheet /gt/gt-delivery/SingleCellBiologyGroup_CT/<A FASTQ DIRECTORY>/* /gt/gt-delivery/SingleCellBiologyGroup_CT/<ANOTHER FASTQ DIRECTORY>/*
```
The shell will expand the globs and pass a list of files to `scbl-utils`. This design was chosen for flexibility - you can exclude or include whatever files you want, however you want. For example, if you want to construct a samplesheet for all libraries in a GT delivery directory except for `25E1-L1` using [find](https://man7.org/linux/man-pages/man1/find.1.html):
```bash
find /gt/gt_delivery/jax/SingleCellBiology_Group_CT/<DELIVERY DIRECTORY>/ ! -name '*25E1-L1*' | xargs scbl-utils samplesheet
```
### Stage a Xenium Delivery
This command is simpler - most of the time, the following will suffice:
```bash
scbl-utils stage-xenium /path/to/xenium_data_directory /path/to/another_xenium_data_directory
```
Currently, this command uses the Unix's `mv` command under the hood, as it can easily handle moving files across network boundaries (instrument files and the staging directory are actually on different physical devices). As a result, the operation is quite slow, so you may want to invoke `scbl-utils stage-xenium` as a background job with `slurm`. However, before each file move, you will be prompted to check that the file renaming is correct - you can skip this prompt for unattended environments (like a `slurm` job) by using the `--yes` option:
```bash
scbl-utils stage-xenium /path/to/xenium_data_directory /path/to/another_xenium_data_directory --yes
```
## To Do
- **Generate post-`nf-tenx` summary data metrics**: Before delivering data to our end-users, we typically create a set of summary CSVs from the `nf-tenx` outputs. This is a monotonous process that is easily automated away.
-
