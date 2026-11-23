use avatar_audio_graph::effects::names::Channel;
use avatar_audio_graph::graph::AudioRequest;

fn check_mute(value: i64) {
    let accepted = AudioRequest::mute(Channel::Agent, value).is_ok();
    assert_eq!(accepted, matches!(value, 0 | 1), "mute {value}");
}

fn check_volume(value: i64) {
    let accepted = AudioRequest::volume(Channel::Human, value);
    assert_eq!(
        accepted.is_ok(),
        (0..=100).contains(&value),
        "volume {value}"
    );
    if let Ok(AudioRequest::Level(level)) = accepted {
        assert_eq!(level.value, format!("{value}%"));
    }
}

#[test]
fn mute_accepts_exactly_zero_and_one_across_a_finite_range() {
    (-1000_i64..=1000).for_each(check_mute);
}

#[test]
fn volume_accepts_exactly_zero_to_one_hundred_and_formats_a_percent() {
    (-1000_i64..=1000).for_each(check_volume);
}
