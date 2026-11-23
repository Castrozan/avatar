use super::room_audio_fakes::*;
use crate::fakes::{TestResult, executable};
use avatar_live::boundaries::Speaker;
use avatar_mac::edge::room_audio::RoomMicrophone;
use std::sync::Arc;
use std::sync::atomic::Ordering;

#[test]
fn every_sample_reaches_the_seat_once_in_order_in_short_chunks() {
    let (wire, levels) = (Wire::default(), Levels::default());
    let mut speaker = speaker(&wire, &levels, &Arc::default());
    let pcm: Vec<u8> = (0..4000_u32)
        .map(|index| u8::try_from(index % 251).unwrap_or(0))
        .collect();
    speaker.play(&pcm);
    assert!(eventually(|| wire.total().len() == pcm.len()));
    assert_eq!(wire.total(), pcm);
    assert!(wire.longest() <= 1920);
}

#[test]
fn clearing_hushes_the_seat_and_closes_the_mouth() {
    let (wire, levels, hushes) = (Wire::default(), Levels::default(), Arc::default());
    let mut speaker = speaker(&wire, &levels, &hushes);
    speaker.play(&vec![0; 1920 * 50]);
    speaker.clear();
    assert_eq!(hushes.load(Ordering::SeqCst), 1);
    assert!(speaker.buffered_seconds().abs() < f64::EPSILON);
    assert_eq!(levels.values().last().map(|level| level.to_bits()), Some(0));
}

#[test]
fn the_mouth_follows_the_loudness_then_closes() {
    let (wire, levels) = (Wire::default(), Levels::default());
    let mut speaker = speaker(&wire, &levels, &Arc::default());
    speaker.play(&[0x00, 0x20].repeat(960));
    assert!(eventually(|| levels.values().contains(&0.25)));
    assert!(eventually(|| levels.values().last() == Some(&0.0)));
}

#[test]
fn closing_closes_the_mouth_and_the_seat() {
    let (wire, levels) = (Wire::default(), Levels::default());
    let mut speaker = speaker(&wire, &levels, &Arc::default());
    speaker.close();
    assert!(levels.closed.load(Ordering::SeqCst));
    assert!(eventually(|| wire.closed.load(Ordering::SeqCst)));
}

#[test]
fn a_refusing_seat_is_remembered_as_the_failure() {
    let (wire, levels) = (Wire::default(), Levels::default());
    wire.refuse.store(true, Ordering::SeqCst);
    let mut speaker = speaker(&wire, &levels, &Arc::default());
    speaker.play(&[0; 100]);
    assert!(eventually(|| speaker.failure().is_some()));
    assert_eq!(speaker.failure().as_deref(), Some("the seat refused"));
}

#[test]
fn the_microphone_yields_whole_chunks_of_the_capture() -> TestResult {
    let directory = tempfile::tempdir()?;
    let capture = executable(directory.path(), "capture", "head -c 6400 /dev/zero")?;
    let command = [capture.display().to_string()];
    let chunks: Vec<Vec<u8>> = RoomMicrophone::start(&command, 16000)?.collect();
    assert_eq!(chunks.len(), 2);
    assert!(chunks.iter().all(|chunk| !chunk.is_empty()));
    assert!(RoomMicrophone::start(&[], 16000).is_err());
    Ok(())
}
