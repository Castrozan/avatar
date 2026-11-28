mod brief;
mod failures;
mod heard;
mod outcomes;
mod script;
mod server;
mod server_thread;

use std::fs;
use std::process::{Output, Stdio};

use heard::Heard;
use server::MockLive;

use crate::{Machine, RUNTIME_DIRECTORY, TestResult, text};

const PAREC: &str = "head -c 4800 /dev/zero; exec sleep 600";
const PACAT: &str = "echo \"$@\" >> \"${0%/*}/../pacat.log\"; cat >> \"${0%/*}/../playback.raw\"";
const A2A: &str = "case \"$1\" in send) echo task-1;; status) echo '{\"state\":\"completed\",\"output\":\"LIVE-ANSWER-1: it is done\"}';; esac";
const HOLDER: &str = r#"{"pane":"pane-1","servant":"servant","heard_through":0,"host":null}"#;

struct Session {
    machine: Machine,
    output: Output,
    heard: Heard,
}

pub(super) fn live_machine() -> Result<Machine, Box<dyn std::error::Error>> {
    live_machine_with(A2A)
}

fn live_machine_with(a2a: &str) -> Result<Machine, Box<dyn std::error::Error>> {
    let machine = Machine::with_tools(&[("parec", PAREC), ("pacat", PACAT), ("a2a", a2a)])?;
    fs::create_dir_all(machine.path(".secrets"))?;
    fs::write(machine.path(".secrets/elevenlabs-api-key"), "test-key\n")?;
    fs::write(machine.runtime().join("addressee.json"), HOLDER)?;
    Ok(machine)
}

fn spawn_live(machine: &Machine, endpoint: &str) -> std::io::Result<std::process::Child> {
    let mut command = machine.command_with_system_tools(&["live", "--endpoint", endpoint]);
    command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
}

fn run_session(a2a: &str) -> Result<Session, Box<dyn std::error::Error>> {
    let machine = live_machine_with(a2a)?;
    let server = MockLive::start()?;
    let child = spawn_live(&machine, &format!("http://{}", server.address))?;
    let output = child.wait_with_output()?;
    let heard = server.heard();
    Ok(Session {
        machine,
        output,
        heard,
    })
}

impl Session {
    fn screen(&self) -> String {
        text(&self.output.stdout)
    }

    fn file(&self, relative: &str) -> String {
        fs::read_to_string(self.machine.path(relative)).unwrap_or_default()
    }
}

#[test]
fn a_session_exits_cleanly_and_announces_itself() -> TestResult {
    let session = run_session(A2A)?;
    let stderr = text(&session.output.stderr);
    assert_eq!(session.output.status.code(), Some(0), "{stderr}");
    let screen = session.screen();
    assert!(screen.contains("elevenlabs-agents live, conversation conv-1"));
    assert!(screen.contains("\"provider\": \"elevenlabs-agents\""));
    Ok(())
}

#[test]
fn a_session_authenticates_with_the_key_file_and_streams_the_microphone() -> TestResult {
    let heard = run_session(A2A)?.heard;
    assert!(!heard.api_keys.is_empty());
    assert!(heard.api_keys.iter().all(|key| key == "test-key"));
    assert!(heard.streamed_audio());
    Ok(())
}

#[test]
fn a_session_logs_the_turns_and_relays_the_agent_answer() -> TestResult {
    let session = run_session(A2A)?;
    let screen = session.screen();
    let logged = session.file(&format!("{RUNTIME_DIRECTORY}/transcript.jsonl"));
    assert!(screen.contains("human: check the build"));
    assert!(screen.contains("task 1 for servant at pane-1: check the build"));
    assert!(logged.contains("check the build"));
    assert!(session.heard.delivered("it is done"));
    Ok(())
}

#[test]
fn a_session_plays_the_voice_to_every_room_sink() -> TestResult {
    let session = run_session(A2A)?;
    assert!(
        session
            .file("pacat.log")
            .contains("--client-name=avatar-live-lip-sync")
    );
    assert_eq!(session.file("playback.raw").len(), 9600);
    Ok(())
}
