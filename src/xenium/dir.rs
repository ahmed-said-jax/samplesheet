use camino::{Utf8Path, Utf8PathBuf};

// The format for a xenium output file, somewhat stupidly, is:
// ├── <DATE>__<SOME STRING>__<RUN ID>
// │   ├── output-<MACHINE ID>__<SLIDE NAME>__<REGION NAME>__<DATE>__<SOME STRING>
//
// This is dumb because we're using the SLIDE NAME instead of the SLIDE ID. The latter is made by us and guaranteed to
// be unique.

struct TopLevel<'a> {
    path: &'a Utf8Path,
    run_id: &'a str,
    sub_dirs: Vec<Utf8PathBuf>,
}

struct SubDir<'a> {
    path: &'a Utf8Path,
    slide_name: &'a str, // This is so stupid
}
