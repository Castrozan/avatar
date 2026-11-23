use avatar_mac::client::{
    SeatPrograms, Speakers, call_capture_exit, capture_command, copy_call_audio, hush_and_leave,
    play_recording, stream_header,
};
use avatar_mac::failure::Failure;
use serde_json::{Value, json};

use super::fake_endpoint::FakeEndpoint;
use crate::support::{assert_same, each, entries, frame_of, hex_bytes, hex_text, parse, strings};
use crate::world::Log;

fn fixture() -> Value {
    parse(include_str!("../fixtures/client.json"))
}

fn program(key: &str) -> String {
    let programs = &fixture()["programs"];
    programs[key].as_str().unwrap_or_default().to_owned()
}

fn programs() -> SeatPrograms {
    let (python, seat) = (program("python"), program("seat"));
    SeatPrograms {
        python,
        seat,
        call_capture: program("call_capture"),
    }
}

fn check_capture(case: &Value) {
    let device = case[0].as_str().unwrap_or_default();
    assert_eq!(capture_command(&programs(), device), strings(&case[1]));
}

fn check_header(case: &Value) {
    let rate = u32::try_from(case[0].as_u64().unwrap_or(0)).unwrap_or(0);
    assert_eq!(stream_header(rate), case[1].as_str().unwrap_or_default());
}

#[test]
fn capture_commands_and_stream_headers_match_python() {
    each(&fixture()["capture"], check_capture);
    each(&fixture()["headers"], check_header);
}

struct Speaker(Log);

impl Speakers for Speaker {
    fn play(&mut self, _recording: &str) -> Result<(), Failure> {
        self.0.push(json!("speakers"));
        Ok(())
    }
}

fn expected_play(python: &Value) -> Value {
    if python[0] == "seat_play" {
        json!(["seat_play", python[1]])
    } else {
        json!("speakers")
    }
}

fn check_play(case: &Value) {
    let log = Log::default();
    let mut fake = FakeEndpoint::new(&log, case["attached"] == true);
    let wav = hex_bytes(case["wav"].as_str().unwrap_or_default());
    let mut speaker = Speaker(log.clone());
    let outcome = play_recording(&mut fake, &mut speaker, "recording", |_| Ok(wav));
    assert_eq!(outcome, Ok(()));
    assert_eq!(log.entries(), vec![expected_play(&case["log"][0])]);
}

#[test]
fn a_recording_plays_through_the_seat_when_attached_and_the_speakers_otherwise() {
    each(&fixture()["plays"], check_play);
}

#[test]
fn terminating_hushes_the_seat_and_exits_cleanly() {
    let log = Log::default();
    let code = hush_and_leave(&mut FakeEndpoint::new(&log, true));
    let seen = json!([log.entries()[0], ["exit", code]]);
    assert_same(&seen, &fixture()["hush_and_leave"]);
}

fn check_capture_copy(case: &Value) {
    let frames = entries(&case["frames"])
        .iter()
        .map(frame_of)
        .collect::<Vec<_>>();
    let mut output = Vec::new();
    assert_eq!(copy_call_audio(frames.into_iter(), &mut output), Ok(()));
    assert_eq!(
        hex_text(&output),
        case["output"].as_str().unwrap_or_default()
    );
}

fn check_exit(case: &Value) {
    assert_eq!(call_capture_exit(&Err(Failure::runtime("closed"))), case[1]);
}

#[test]
fn call_audio_is_copied_frame_by_frame_and_failures_exit_one() {
    each(&fixture()["captures"], check_capture_copy);
    each(&fixture()["capture_exits"], check_exit);
}
