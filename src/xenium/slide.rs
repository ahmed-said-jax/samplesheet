#[derive(bon::Builder, Debug)]
pub(super) struct Slide<'a> {
    pub(super) id: &'a str,
    pub(super) name: &'a str,
    pub(super) run_id: &'a str,
    pub(super) lab_name: &'a str,
}
