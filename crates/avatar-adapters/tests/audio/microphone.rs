use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, Instant};

use avatar_adapters::commands::MicrophoneChunks;

use crate::tools::{TestResult, Tools};

const PARECORD: &str = "echo \"$@\" > \"${0%/*}/args\"; head -c 8000 /dev/zero";
const ARGUMENTS: &str = "--device=mic.monitor --raw --rate=24000 --channels=1 --format=s16le --latency-msec=40 --client-name=avatar-live-microphone\n";

fn start(tools: &Tools, rate: u32) -> Result<MicrophoneChunks, std::io::Error> {
    MicrophoneChunks::start(&tools.programs(), rate, "mic.monitor")
}

#[test]
fn it_yields_whole_hundred_millisecond_chunks_and_drops_the_incomplete_tail() -> TestResult {
    let tools = Tools::with(&[("parec", PARECORD)])?;
    let lengths: Vec<usize> = start(&tools, 16000)?.map(|chunk| chunk.len()).collect();
    assert_eq!(lengths, [3200, 3200]);
    Ok(())
}

#[test]
fn it_records_with_the_python_arguments() -> TestResult {
    let tools = Tools::with(&[("parec", PARECORD)])?;
    let mut chunks = start(&tools, 24000)?;
    assert_eq!(chunks.next().map(|chunk| chunk.len()), Some(4800));
    assert_eq!(tools.read("args"), ARGUMENTS);
    Ok(())
}

fn recorder_pid(tools: &Tools) -> String {
    let deadline = Instant::now() + Duration::from_secs(5);
    while tools.read("pid").trim().is_empty() && Instant::now() < deadline {
        sleep(Duration::from_millis(10));
    }
    tools.read("pid").trim().to_owned()
}

#[test]
fn dropping_the_stream_terminates_a_recorder_that_never_ends() -> TestResult {
    let tools = Tools::with(&[("parec", "echo $$ > \"${0%/*}/pid\"; exec sleep 30")])?;
    let chunks = start(&tools, 16000)?;
    let process = format!("/proc/{}", recorder_pid(&tools));
    assert!(Path::new(&process).exists());
    drop(chunks);
    assert!(!Path::new(&process).exists());
    Ok(())
}
