use avatar_listening::numeric::count_as_f64;
use avatar_listening::speech_segmenter::{FRAME_SECONDS, TRAILING_SILENCE_FRAMES};
use proptest::prelude::*;

use super::harness::{Harness, Kind, frame_of, is_loud};

fn piece_strategy() -> impl Strategy<Value = (Kind, usize)> {
    (
        prop_oneof![Just(Kind::Speech), Just(Kind::Noise)],
        0..TRAILING_SILENCE_FRAMES,
    )
}

fn moment_strategy() -> impl Strategy<Value = Vec<(Kind, usize)>> {
    proptest::collection::vec(piece_strategy(), 1..15)
}

fn frames_of(pieces: &[(Kind, usize)]) -> Vec<Kind> {
    pieces
        .iter()
        .flat_map(|&(sound, pause)| {
            std::iter::once(sound).chain(std::iter::repeat_n(Kind::Silence, pause))
        })
        .collect()
}

fn expected_stop(frames: &[Kind]) -> Option<f64> {
    let fifth = frames
        .iter()
        .enumerate()
        .filter(|(_, kind)| **kind == Kind::Speech)
        .nth(4)?
        .0;
    Some(count_as_f64(fifth + 1) * FRAME_SECONDS)
}

fn hear_kinds(harness: &mut Harness, kinds: &[Kind]) -> Vec<Option<f64>> {
    kinds
        .iter()
        .map(|&kind| {
            let frame = frame_of(kind);
            harness.barge.hear(&frame, is_loud(&frame))
        })
        .collect()
}

fn moment_stop(harness: &mut Harness, frames: &[Kind]) -> Option<f64> {
    harness.barge.begin_segment();
    let padded: Vec<Kind> = frames
        .iter()
        .copied()
        .chain(std::iter::repeat_n(Kind::Silence, TRAILING_SILENCE_FRAMES))
        .collect();
    hear_kinds(harness, &padded).into_iter().flatten().next()
}

proptest! {
    #[test]
    fn a_reply_stops_at_the_fifth_speech_frame_of_each_moment_and_only_then(
        moments in proptest::collection::vec(moment_strategy(), 1..5),
    ) {
        let mut harness = Harness::start();
        for pieces in &moments {
            let frames = frames_of(pieces);
            prop_assert_eq!(moment_stop(&mut harness, &frames), expected_stop(&frames));
        }
    }

    #[test]
    fn nothing_is_sent_while_no_reply_is_spoken(
        kinds in proptest::collection::vec(prop_oneof![Just(Kind::Speech), Just(Kind::Noise), Just(Kind::Silence)], 0..400),
    ) {
        let mut harness = Harness::start();
        harness.cells.speaking.set(false);
        for kind in kinds {
            prop_assert!(!harness.hear(&frame_of(kind)));
        }
        prop_assert_eq!(harness.cells.judged.get(), 0);
    }

    #[test]
    fn sound_that_is_not_loud_speech_never_stops_a_reply(
        kinds in proptest::collection::vec(prop_oneof![Just(Kind::Noise), Just(Kind::Quiet), Just(Kind::Silence)], 0..600),
    ) {
        let mut harness = Harness::start();
        for kind in kinds {
            prop_assert!(!harness.hear(&frame_of(kind)));
        }
        prop_assert_eq!(harness.cells.stops.get(), 0);
    }
}
