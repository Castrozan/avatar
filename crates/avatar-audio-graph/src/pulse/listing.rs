use serde::Deserialize;

use super::control::PulseError;
use super::module_record::ModuleRecord;

#[derive(Deserialize)]
struct NamedNode {
    name: String,
}

#[must_use]
pub fn parse_modules(output: &str) -> Vec<ModuleRecord> {
    output.lines().filter_map(parse_module_line).collect()
}

fn parse_module_line(line: &str) -> Option<ModuleRecord> {
    let fields: Vec<&str> = line.splitn(4, '\t').collect();
    if fields.len() < 3 || !is_decimal(fields[0]) {
        return None;
    }
    let index = fields[0].parse::<u32>().ok()?;
    Some(ModuleRecord::new(index, fields[1], fields[2].to_owned()))
}

fn is_decimal(text: &str) -> bool {
    !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit())
}

pub fn node_names(output: &str) -> Result<Vec<String>, PulseError> {
    let nodes: Vec<NamedNode> = serde_json::from_str(output)
        .map_err(|error| PulseError::new(format!("invalid node listing: {error}")))?;
    Ok(nodes.into_iter().map(|node| node.name).collect())
}

#[must_use]
pub fn sink_identifier(sinks_listing: &str, sink_name: &str) -> Option<String> {
    sinks_listing
        .lines()
        .map(|line| line.split('\t').collect::<Vec<&str>>())
        .find(|fields| fields.len() > 1 && fields[1] == sink_name)
        .map(|fields| fields[0].to_owned())
}

#[must_use]
pub fn sink_input_identifiers(inputs_listing: &str, sink_identifier: &str) -> Vec<String> {
    inputs_listing
        .lines()
        .map(|line| line.split('\t').collect::<Vec<&str>>())
        .filter(|fields| fields.len() > 1 && fields[1] == sink_identifier)
        .map(|fields| fields[0].to_owned())
        .collect()
}
