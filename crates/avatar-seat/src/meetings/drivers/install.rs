use std::path::Path;

use super::{COMMAND_TIMEOUT_SECONDS, Driver, DriverEdges, DriverPlan};
use crate::edges::EdgeFailure;
use crate::meetings::run_checked;
use crate::support::words::joined_words;

fn text_of(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn name_of(path: &Path) -> String {
    path.file_name()
        .map_or_else(String::new, |name| text_of(name.as_ref()))
}

fn is_installed(
    edges: &DriverEdges<'_>,
    packaged: &Path,
    installed: &Path,
) -> Result<bool, EdgeFailure> {
    if !edges.store.exists(installed) {
        return Ok(false);
    }
    let paths = [text_of(packaged), text_of(installed)];
    let command = [joined_words(&[&["/usr/bin/diff", "-rq"]]), paths.to_vec()].concat();
    let comparison = edges
        .processes
        .run(&command, COMMAND_TIMEOUT_SECONDS, true)?;
    Ok(comparison.code == 0)
}

fn missing_driver(
    edges: &DriverEdges<'_>,
    directory: &Path,
    packaged: &Path,
) -> Result<Option<Driver>, EdgeFailure> {
    let installed = directory.join(name_of(packaged));
    let present = is_installed(edges, packaged, &installed)?;
    Ok((!present).then(|| (packaged.to_path_buf(), installed)))
}

fn missing_drivers(edges: &DriverEdges<'_>, plan: &DriverPlan) -> Result<Vec<Driver>, EdgeFailure> {
    let found: Result<Vec<_>, _> = (plan.packaged.iter())
        .map(|packaged| missing_driver(edges, &plan.directory, packaged))
        .collect();
    Ok(found?.into_iter().flatten().collect())
}

fn restart_audio_server(edges: &DriverEdges<'_>) -> Result<(), EdgeFailure> {
    let command = joined_words(&[&["/usr/bin/killall", "coreaudiod"]]);
    run_checked(edges.processes, &command, COMMAND_TIMEOUT_SECONDS, false).map(drop)
}

fn announce(edges: &DriverEdges<'_>, missing: &[Driver]) {
    let names: Vec<String> = missing
        .iter()
        .map(|(_, installed)| name_of(installed))
        .collect();
    edges.output.line(&format!(
        "Installed the call audio drivers {}",
        names.join(", ")
    ));
}

pub fn install_call_audio_drivers(
    edges: &DriverEdges<'_>,
    plan: &DriverPlan,
) -> Result<(), EdgeFailure> {
    let missing = missing_drivers(edges, plan)?;
    for (packaged, installed) in &missing {
        edges.store.replace(packaged, installed)?;
    }
    if missing.is_empty() {
        return Ok(());
    }
    restart_audio_server(edges)?;
    announce(edges, &missing);
    Ok(())
}
