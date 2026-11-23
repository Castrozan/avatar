use avatar_listening::barge_in::reply_link::{ReplyLinkState, ServerNotice};
use serde_json::{Value, json};

use crate::support::{Outcome, fixture, flag, rows};

fn notice_of(message: &Value) -> Result<ServerNotice, Box<dyn std::error::Error>> {
    Ok(ServerNotice::from_message(message).ok_or("notice without a flag")?)
}

fn replay_sequence(case: &Value) -> Outcome {
    let mut state = ReplyLinkState::default();
    let speaking = rows(&case["speaking"])?;
    for (message, expected) in rows(&case["messages"])?.iter().zip(speaking) {
        state.hear_notice(&notice_of(message)?);
        let expected_flag = expected.as_i64().ok_or("flag")? == 1;
        assert_eq!(state.speaking(), expected_flag);
    }
    Ok(())
}

#[test]
fn every_oracle_message_sequence_matches_the_python_reply_link() -> Outcome {
    let document = fixture("barge_in/fixtures/reply_link.json")?;
    for case in rows(&document["cases"])? {
        replay_sequence(case)?;
    }
    Ok(())
}

fn check_speaking_value(case: &Value) -> Outcome {
    let message = json!({"type": "speechActivity", "speaking": case["speaking"]});
    let expected = ServerNotice::SpeechActivity {
        speaking: flag(&case["flag"])?,
    };
    assert_eq!(notice_of(&message)?, expected);
    Ok(())
}

#[test]
fn every_oracle_speaking_value_follows_the_python_truthiness() -> Outcome {
    let document = fixture("barge_in/fixtures/speaking_flag.json")?;
    let cases = rows(&document["cases"])?;
    assert_eq!(cases.len(), 17);
    cases.iter().try_for_each(check_speaking_value)
}

#[test]
fn a_speech_activity_without_a_flag_is_no_notice_at_all() -> Outcome {
    let document = fixture("barge_in/fixtures/speaking_flag.json")?;
    assert_eq!(document["other"][1]["outcome"], "KeyError");
    assert_eq!(
        ServerNotice::from_message(&json!({"type": "speechActivity"})),
        None
    );
    Ok(())
}

#[test]
fn a_lost_server_means_nothing_is_speaking() {
    let mut state = ReplyLinkState::default();
    state.hear_notice(&ServerNotice::SpeechActivity { speaking: true });
    state.lost_server();
    assert!(!state.speaking());
}
