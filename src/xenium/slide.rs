#[derive(bon::Builder, Debug)]
pub(super) struct Slide<'a> {
    lab_name: &'a str,
    run_id: &'a str,
    id: &'a str,
    name: &'a str,
}
