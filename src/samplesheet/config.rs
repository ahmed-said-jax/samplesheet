use std::collections::HashMap;

use camino::Utf8PathBuf;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub(super) species_reference_path: HashMap<String, HashMap<String, Utf8PathBuf>>,
    pub(super) chemistry_program: HashMap<String, (String, String, String)>,
    pub(super) species_probe_set: HashMap<String, Utf8PathBuf>,
}
