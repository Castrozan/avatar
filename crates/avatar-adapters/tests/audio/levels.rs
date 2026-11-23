use std::fs;

use avatar_adapters::commands::{AudioEnvironment, run_audio};
use avatar_audio_graph::effects::names::Channel;
use avatar_audio_graph::graph::AudioRequest;
use serde_json::json;

use crate::mock::{MockServer, closed_port};
use crate::status::environment;
use crate::tools::{TestResult, Tools};

const VOLUME_CALL: &str = "set-sink-volume AvatarHuman 40%\n";
const PACTL: &str = "echo \"$@\" >> \"${0%/*}/log\"; case \"$*\" in \"list short modules\") printf '10\\tmodule-null-sink\\tsink_name=AvatarMix\\n';; \"--format=json list sinks\") echo '[]';; esac";

fn running(tools: &Tools, seat_port: u16) -> Result<AudioEnvironment, std::io::Error> {
    let environment = environment(tools, seat_port)?;
    let module =
        json!({"index": 10, "name": "module-null-sink", "argument": "sink_name=AvatarMix"});
    let state = json!({"modules": [module], "ready": true});
    fs::write(
        environment.runtime_directory.join("audio.json"),
        state.to_string(),
    )?;
    Ok(environment)
}

fn refusal(
    environment: &AudioEnvironment,
    request: Result<AudioRequest, avatar_audio_graph::errors::AudioGraphError>,
) -> String {
    let outcome = run_audio(environment, || request);
    outcome
        .err()
        .map(|error| error.to_string())
        .unwrap_or_default()
}

type World = (MockServer, Tools, AudioEnvironment);

fn world() -> Result<World, Box<dyn std::error::Error>> {
    let seat = MockServer::start(200, "")?;
    let tools = Tools::with(&[("pactl", PACTL)])?;
    let environment = running(&tools, seat.port)?;
    Ok((seat, tools, environment))
}

#[test]
fn muting_sets_the_sink_and_tells_the_seat() -> TestResult {
    let (seat, tools, environment) = world()?;
    run_audio(&environment, || AudioRequest::mute(Channel::Agent, 1))?;
    assert!(tools.read("log").contains("set-sink-mute AvatarMic 1\n"));
    assert_eq!(seat.paths(), ["/levels"]);
    Ok(())
}

#[test]
fn a_volume_change_sets_a_percentage_and_tells_the_seat() -> TestResult {
    let (seat, tools, environment) = world()?;
    run_audio(&environment, || AudioRequest::volume(Channel::Human, 40))?;
    assert!(tools.read("log").contains(VOLUME_CALL));
    assert_eq!(seat.paths(), ["/levels"]);
    Ok(())
}

#[test]
fn other_actions_leave_the_seat_alone() -> TestResult {
    let seat = MockServer::start(200, "")?;
    let tools = Tools::with(&[("pactl", PACTL)])?;
    run_audio(&running(&tools, seat.port)?, || Ok(AudioRequest::Status))?;
    assert!(seat.paths().is_empty());
    Ok(())
}

#[test]
fn a_dead_seat_does_not_fail_the_command() -> TestResult {
    let tools = Tools::with(&[("pactl", PACTL)])?;
    let environment = running(&tools, closed_port()?)?;
    assert!(run_audio(&environment, || AudioRequest::mute(Channel::Agent, 0)).is_ok());
    Ok(())
}

#[test]
fn invalid_levels_are_refused_with_the_python_messages() -> TestResult {
    let tools = Tools::with(&[("pactl", PACTL)])?;
    let environment = running(&tools, 1)?;
    assert_eq!(
        refusal(&environment, AudioRequest::mute(Channel::Agent, 2)),
        "Mute must be 0 or 1"
    );
    let volume = refusal(&environment, AudioRequest::volume(Channel::Agent, 101));
    assert_eq!(volume, "Volume must be between 0 and 100");
    Ok(())
}

#[test]
fn a_stopped_graph_refuses_level_changes() -> TestResult {
    let tools = Tools::with(&[("pactl", PACTL)])?;
    let environment = environment(&tools, 1)?;
    let message = refusal(&environment, AudioRequest::mute(Channel::Agent, 1));
    assert_eq!(message, "Avatar audio is not running with its owned graph");
    Ok(())
}
