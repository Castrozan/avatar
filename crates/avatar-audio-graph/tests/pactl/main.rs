mod properties;

use avatar_audio_graph::pulse::listing::{
    node_names, parse_modules, sink_identifier, sink_input_identifiers,
};
use avatar_audio_graph::pulse::module_record::ModuleRecord;
use serde::Deserialize;

const FIXTURE: &str = include_str!("fixtures/listing.json");

#[derive(Deserialize)]
struct ListingFixture {
    parse_modules: Vec<ParseCase>,
    active_playbacks: Vec<PlaybackCase>,
}

#[derive(Deserialize)]
struct ParseCase {
    output: String,
    modules: Vec<ModuleRecord>,
}

#[derive(Deserialize)]
struct PlaybackCase {
    sinks: String,
    sink_inputs: String,
    sink: String,
    active: Vec<String>,
}

fn active_playbacks(sinks: &str, inputs: &str, sink: &str) -> Vec<String> {
    sink_identifier(sinks, sink)
        .map(|identifier| sink_input_identifiers(inputs, &identifier))
        .unwrap_or_default()
}

fn fixture() -> Result<ListingFixture, String> {
    serde_json::from_str(FIXTURE).map_err(|error| error.to_string())
}

fn check_parse_case(case: &ParseCase) {
    assert_eq!(
        parse_modules(&case.output),
        case.modules,
        "{:?}",
        case.output
    );
}

fn check_playback_case(case: &PlaybackCase) {
    assert_eq!(
        active_playbacks(&case.sinks, &case.sink_inputs, &case.sink),
        case.active
    );
}

#[test]
fn parse_modules_matches_every_python_listing() -> Result<(), String> {
    let fixture = fixture()?;
    assert_eq!(fixture.parse_modules.len(), 8);
    fixture.parse_modules.iter().for_each(check_parse_case);
    Ok(())
}

#[test]
fn active_playbacks_matches_every_python_listing() -> Result<(), String> {
    let fixture = fixture()?;
    fixture
        .active_playbacks
        .iter()
        .for_each(check_playback_case);
    Ok(())
}

#[test]
fn node_names_reads_the_json_listing_and_ignores_other_fields() {
    let listing =
        r#"[{"index":3,"name":"AvatarMic","driver":"x"},{"index":4,"name":"AvatarHuman"}]"#;
    let names = node_names(listing).map_err(|error| error.to_string());
    assert_eq!(
        names,
        Ok(vec!["AvatarMic".to_owned(), "AvatarHuman".to_owned()])
    );
}
