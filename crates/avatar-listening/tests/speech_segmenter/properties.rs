use super::{FRAME_BYTES, frame_of};

const POOL: [[i16; 2]; 12] = [
    [0, 0],
    [100, -100],
    [299, 300],
    [300, -300],
    [3000, -3000],
    [3001, 3001],
    [200, 500],
    [0, 400],
    [-50, 50],
    [250, 350],
    [2000, 0],
    [0, 0],
];
use avatar_listening::speech_segmenter::{
    MAXIMUM_SEGMENT_FRAMES, MINIMUM_SPEECH_FRAMES, SpeechSegmenter,
};
use proptest::prelude::*;

fn check_rules(segment: &[u8]) -> Result<(), TestCaseError> {
    let frame_count = segment.len() / FRAME_BYTES;
    prop_assert_eq!(segment.len() % FRAME_BYTES, 0);
    prop_assert!(frame_count >= MINIMUM_SPEECH_FRAMES);
    prop_assert!(frame_count <= MAXIMUM_SEGMENT_FRAMES);
    Ok(())
}

proptest! {
    #[test]
    fn returned_segments_are_whole_frames_within_the_speech_and_length_rules(
        choices in proptest::collection::vec(0..POOL.len(), 1..700),
    ) {
        let mut segmenter = SpeechSegmenter::default();
        let segments: Vec<Vec<u8>> = choices
            .iter()
            .filter_map(|&choice| segmenter.feed(frame_of(POOL[choice])))
            .collect();
        for segment in &segments {
            check_rules(segment)?;
        }
    }

    #[test]
    fn silence_never_opens_a_segment(length in 0_usize..400) {
        let mut segmenter = SpeechSegmenter::default();
        for _ in 0..length {
            prop_assert!(segmenter.feed(frame_of([0, 0])).is_none());
        }
        prop_assert!(!segmenter.active());
    }
}
