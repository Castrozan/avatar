use std::fs;
use std::io::ErrorKind;
use std::thread::sleep;
use std::time::{Duration, Instant};

use avatar_adapters::commands::{PacatSpeaker, RoomSink};
use avatar_live::boundaries::Speaker;

use crate::tools::{TestResult, Tools};

const RECORD: &str = "echo \"$@\" > \"${0%/*}/args\"; cat >> \"${0%/*}/pcm\"";
const DEFAULT_ARGUMENTS: &str = "--playback --raw --rate=24000 --channels=1 --format=s16le --latency-msec=80 --client-name=avatar-live\n";

fn until(mut condition: impl FnMut() -> bool) -> bool {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !condition() && Instant::now() < deadline {
        sleep(Duration::from_millis(10));
    }
    condition()
}

fn start(tools: &Tools, rate: u32, sink: RoomSink) -> Result<PacatSpeaker, std::io::Error> {
    PacatSpeaker::start(&tools.programs(), rate, &[sink])
}

#[test]
fn pcm_reaches_pacat_whole_and_in_order() -> TestResult {
    let tools = Tools::with(&[("pacat", RECORD)])?;
    let mut speaker = start(&tools, 24000, RoomSink::Default)?;
    let pcm: Vec<u8> = (0..=255).cycle().take(10_000).collect();
    speaker.play(&pcm);
    assert!(until(
        || fs::read(tools.path().join("pcm")).is_ok_and(|read| read.len() == 10_000)
    ));
    assert_eq!(fs::read(tools.path().join("pcm"))?, pcm);
    Ok(())
}

#[test]
fn the_default_sink_uses_the_python_arguments() -> TestResult {
    let tools = Tools::with(&[("pacat", RECORD)])?;
    let _speaker = start(&tools, 24000, RoomSink::Default)?;
    assert!(until(|| tools.path().join("args").exists()));
    assert!(until(|| tools.read("args") == DEFAULT_ARGUMENTS));
    Ok(())
}

#[test]
fn the_lip_sync_sink_gets_its_own_device_and_client_name() -> TestResult {
    let tools = Tools::with(&[("pacat", RECORD)])?;
    let _speaker = start(&tools, 16000, RoomSink::LipSync)?;
    assert!(until(|| tools.read("args").ends_with(
        "--client-name=avatar-live-lip-sync --device=AvatarLipSync\n"
    )));
    Ok(())
}

#[test]
fn buffered_seconds_count_hundred_millisecond_chunks_and_clear_empties_them() -> TestResult {
    let tools = Tools::with(&[("pacat", "exec sleep 5")])?;
    let mut speaker = start(&tools, 16000, RoomSink::Default)?;
    speaker.play(&vec![0; 3200 * 300]);
    assert!(until(|| speaker.buffered_seconds() > 20.0));
    speaker.clear();
    assert!(speaker.buffered_seconds() < f64::EPSILON);
    Ok(())
}

#[test]
fn a_pacat_that_dies_leaves_a_broken_pipe_failure() -> TestResult {
    let tools = Tools::with(&[("pacat", "exit 0")])?;
    let mut speaker = start(&tools, 16000, RoomSink::Default)?;
    assert!(until(|| {
        speaker.play(&[0; 6400]);
        speaker.failure().is_some()
    }));
    assert_eq!(speaker.failure(), Some(ErrorKind::BrokenPipe));
    Ok(())
}
