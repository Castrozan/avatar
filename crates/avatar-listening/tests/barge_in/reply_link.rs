use avatar_listening::barge_in::reply_link::{ReplyLinkState, ServerNotice};
use serde_json::Value;

use crate::support::{Outcome, fixture, flag, rows, text};

fn notice_of(message: &Value) -> Result<ServerNotice, Box<dyn std::error::Error>> {
    if text(&message["type"])? != "speechActivity" {
        return Ok(ServerNotice::Other);
    }
    Ok(ServerNotice::SpeechActivity {
        speaking: flag(&message["speaking"])?,
    })
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

#[test]
fn a_lost_server_means_nothing_is_speaking() {
    let mut state = ReplyLinkState::default();
    state.hear_notice(&ServerNotice::SpeechActivity { speaking: true });
    state.lost_server();
    assert!(!state.speaking());
}
