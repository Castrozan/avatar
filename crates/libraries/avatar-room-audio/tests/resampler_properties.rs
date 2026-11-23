use std::num::NonZeroUsize;

use avatar_room_audio::resampler::LinearResampler;
use proptest::prelude::*;

const RATES: [usize; 6] = [8000, 16000, 22050, 24000, 44100, 48000];

fn rate() -> impl Strategy<Value = usize> {
    proptest::sample::select(RATES.to_vec())
}

fn samples() -> impl Strategy<Value = Vec<i16>> {
    prop::collection::vec(any::<i16>(), 1..400)
}

fn resampler(source: usize, target: usize) -> LinearResampler {
    let to_rate = |value: usize| NonZeroUsize::new(value).unwrap_or(NonZeroUsize::MIN);
    LinearResampler::new(to_rate(source), to_rate(target))
}

fn ceiling_division(numerator: usize, denominator: usize) -> usize {
    numerator.div_ceil(denominator)
}

proptest! {
    #[test]
    fn the_length_follows_the_rate_ratio_holding_back_only_the_last_sample(
        source in rate(), target in rate(), input in samples()
    ) {
        let produced = resampler(source, target).resample(&input);
        let expected = ceiling_division((input.len() - 1) * target, source);
        prop_assert_eq!(produced.len(), expected);
    }

    #[test]
    fn cutting_the_stream_in_two_changes_nothing(
        source in rate(), target in rate(), input in samples(), cut in any::<usize>()
    ) {
        let whole = resampler(source, target).resample(&input);
        let (head, tail) = input.split_at(cut % (input.len() + 1));
        let mut streaming = resampler(source, target);
        let mut pieces = streaming.resample(head);
        pieces.extend(streaming.resample(tail));
        prop_assert_eq!(pieces, whole);
    }

    #[test]
    fn the_output_never_leaves_the_range_of_the_input(
        source in rate(), target in rate(), input in samples()
    ) {
        let lowest = input.iter().copied().min().unwrap_or(i16::MIN);
        let highest = input.iter().copied().max().unwrap_or(i16::MAX);
        let produced = resampler(source, target).resample(&input);
        prop_assert!(produced.iter().all(|sample| (lowest..=highest).contains(sample)));
    }

    #[test]
    fn a_constant_signal_stays_exactly_constant(
        source in rate(), target in rate(), level in any::<i16>(), length in 1usize..400
    ) {
        let produced = resampler(source, target).resample(&vec![level; length]);
        prop_assert!(produced.iter().all(|sample| *sample == level));
    }
}
