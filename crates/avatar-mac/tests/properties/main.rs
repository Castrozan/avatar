use avatar_mac::client::wav_samples;
use avatar_mac::frame::{Frame, whole_samples};
use avatar_mac::relay::{CallRelay, PageAdmission};
use avatar_mac::seat::{CHUNK_BYTES, PacingStep, SpeechPacing, forwarded_audio};
use proptest::prelude::*;

fn word(value: usize) -> [u8; 4] {
    u32::try_from(value).unwrap_or(0).to_le_bytes()
}

fn format_chunk(channels: u16, width: u16) -> Vec<u8> {
    let block = channels * width;
    let rate = 24000u32;
    let head = [1u16.to_le_bytes(), channels.to_le_bytes()].concat();
    let middle = [rate.to_le_bytes(), (rate * u32::from(block)).to_le_bytes()].concat();
    let tail = [block.to_le_bytes(), (width * 8).to_le_bytes()].concat();
    [head, middle, tail].concat()
}

fn wave(channels: u16, width: u16, data: &[u8]) -> Vec<u8> {
    let mut body = [
        b"WAVEfmt ".to_vec(),
        word(16).to_vec(),
        format_chunk(channels, width),
    ]
    .concat();
    body.extend([b"data".to_vec(), word(data.len()).to_vec(), data.to_vec()].concat());
    [b"RIFF".to_vec(), word(body.len()).to_vec(), body].concat()
}

proptest! {
    #[test]
    fn whole_frames_survive_a_wave_round_trip(
        channels in 1u16..3,
        width in 1u16..5,
        frames in proptest::collection::vec(any::<u8>(), 0..64),
    ) {
        let block = usize::from(channels * width);
        let data = &frames[..frames.len() / block * block];
        prop_assert_eq!(wav_samples(&wave(channels, width, data)), Ok(data.to_vec()));
    }

    #[test]
    fn forwarded_audio_is_the_whole_samples_of_binary_frames_only(
        pcm in proptest::collection::vec(any::<u8>(), 0..40),
        present in any::<bool>(),
    ) {
        let forwarded = forwarded_audio(&Frame::Binary(pcm.clone()), present);
        prop_assert_eq!(forwarded, present.then(|| whole_samples(&pcm).to_vec()));
        prop_assert_eq!(forwarded_audio(&Frame::Text("x".into()), present), None);
    }

    #[test]
    fn unhushed_speech_is_sent_whole_in_order_in_bounded_chunks(length in 0usize..6000) {
        let mut pacing = SpeechPacing::new(length, 0.0);
        let mut next_start = 0;
        while let PacingStep::Send { start, end } = pacing.next(false) {
            prop_assert_eq!(start, next_start);
            prop_assert!(end - start <= CHUNK_BYTES && end > start);
            prop_assert!(pacing.pause_after_send(0.0) >= 0.0);
            next_start = end;
        }
        prop_assert_eq!(next_start, length);
    }

    #[test]
    fn the_relay_never_admits_two_pages_at_once(leaves in proptest::collection::vec(any::<bool>(), 0..30)) {
        let mut relay = CallRelay::default();
        let mut carrying = 0u32;
        for leaving in leaves {
            match (leaving, relay.admit_page()) {
                (true, PageAdmission::Carry) => { relay.page_left(); }
                (false, PageAdmission::Carry) => carrying += 1,
                (_, PageAdmission::Refuse { .. }) => prop_assert!(carrying > 0),
            }
            prop_assert!(carrying <= 1);
        }
    }
}
