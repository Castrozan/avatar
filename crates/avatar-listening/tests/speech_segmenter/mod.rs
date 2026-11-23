use crate::support::{Outcome, count, number, optional_count, replay_cases, rows, text};
mod properties;

use avatar_listening::speech_segmenter::{
    MAXIMUM_SEGMENT_FRAMES, MINIMUM_SPEECH_FRAMES, SpeechSegmenter,
};
use serde_json::Value;

const FRAME_BYTES: usize = 1024;
pub(super) fn frame_of(pair: [i16; 2]) -> Vec<u8> {
    (0..512)
        .flat_map(|index| pair[index % 2].to_le_bytes())
        .collect()
}

fn pair_of(value: &Value) -> Result<[i16; 2], Box<dyn std::error::Error>> {
    let first = i16::try_from(value[0].as_i64().ok_or("sample")?)?;
    let second = i16::try_from(value[1].as_i64().ok_or("sample")?)?;
    Ok([first, second])
}

fn frames_of(case: &Value) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
    rows(&case["frames"])?
        .iter()
        .map(|value| Ok(frame_of(pair_of(value)?)))
        .collect()
}

fn expected_segment(frames: &[Vec<u8>], end: usize, frame_count: usize) -> Vec<u8> {
    frames[end + 1 - frame_count..=end].concat()
}

fn check_content(
    frames: &[Vec<u8>],
    index: usize,
    segment: &[u8],
    frame_count: usize,
    label: &str,
) {
    assert_eq!(
        segment,
        expected_segment(frames, index, frame_count),
        "{label} content"
    );
}

fn check_segment(
    frames: &[Vec<u8>],
    index: usize,
    returned: Option<&Vec<u8>>,
    step: &Value,
    name: &str,
) -> Outcome {
    let label = format!("{name} step {index}");
    let frame_count = returned.map(|segment| segment.len() / FRAME_BYTES);
    assert_eq!(frame_count, optional_count(&step[0])?, "{label}");
    if let (Some(segment), Some(total)) = (returned, frame_count) {
        check_content(frames, index, segment, total, &label);
    }
    Ok(())
}

fn check_state(segmenter: &SpeechSegmenter, step: &Value, label: &str) -> Outcome {
    assert_eq!(segmenter.active(), step[1] == 1, "{label} active");
    assert_eq!(segmenter.loud(), step[2] == 1, "{label} loud");
    let closing = number(&step[3])?;
    assert_eq!(
        segmenter.closing_silence_seconds().to_bits(),
        closing.to_bits(),
        "{label} closing"
    );
    Ok(())
}

fn replay(case: &Value) -> Outcome {
    let name = text(&case["name"])?;
    let frames = frames_of(case)?;
    let mut segmenter = SpeechSegmenter::default();
    for (index, (frame, step)) in frames.iter().zip(rows(&case["steps"])?).enumerate() {
        let returned = segmenter.feed(frame.clone());
        check_segment(&frames, index, returned.as_ref(), step, name)?;
        check_state(&segmenter, step, &format!("{name} step {index}"))?;
    }
    Ok(())
}

#[test]
fn every_oracle_case_replays_the_python_segmenter() -> Outcome {
    replay_cases(
        "speech_segmenter/fixtures/segmenter.json",
        "cases",
        11,
        replay,
    )
}

#[test]
fn the_python_frame_thresholds_are_the_documented_counts() -> Outcome {
    assert_eq!(MINIMUM_SPEECH_FRAMES, count(&Value::from(9))?);
    assert_eq!(MAXIMUM_SEGMENT_FRAMES, 375);
    Ok(())
}
