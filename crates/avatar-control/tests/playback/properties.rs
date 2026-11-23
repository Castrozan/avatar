use avatar_control::playback::{AudioPlayer, ProgramAudioPlayer, PulseSinkPlayer};
use proptest::prelude::*;

use super::support::{CLIP_PATH, Scratch, block_on, clip, within_limit};

fn recorded_program_arguments(
    arguments: Vec<String>,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let scratch = Scratch::new("property_program_arguments")?;
    let program = scratch.fake_program("avatar-seat-play", "")?;
    let player = ProgramAudioPlayer::new(program, arguments);
    block_on(async { within_limit(player.play(clip()).finished()).await })??;
    scratch.recorded_arguments().map_err(Into::into)
}

fn recorded_sink_arguments(sink: String) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let scratch = Scratch::new("property_sink_name")?;
    let program = scratch.fake_program("paplay", "")?;
    let player = PulseSinkPlayer::new(program, sink, "/run/user/1000");
    block_on(async { within_limit(player.play(clip()).finished()).await })??;
    scratch.recorded_arguments().map_err(Into::into)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(12))]

    #[test]
    fn program_arguments_precede_the_audio_path_verbatim(
        arguments in proptest::collection::vec("[^\u{0}]{0,12}", 0..4),
    ) {
        let mut expected = arguments.clone();
        expected.push(CLIP_PATH.to_owned());
        let recorded = recorded_program_arguments(arguments)
            .map_err(|error| TestCaseError::fail(error.to_string()))?;
        prop_assert_eq!(recorded, expected);
    }

    #[test]
    fn sink_name_reaches_the_device_argument_verbatim(sink in "[A-Za-z0-9_.:-]{1,24}") {
        let recorded = recorded_sink_arguments(sink.clone())
            .map_err(|error| TestCaseError::fail(error.to_string()))?;
        prop_assert_eq!(
            recorded,
            vec!["--client-name", "avatar-speech", "--device", sink.as_str(), CLIP_PATH]
        );
    }
}
