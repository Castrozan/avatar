use std::error::Error;
use std::net::SocketAddr;

use avatar_live::client::{LiveClient, SessionRequest};
use avatar_live::session::{initiation, instructions};
use proptest::prelude::*;
use serde_json::{Value, json};

use super::request_for;
use super::server::start;
use crate::support::{Outcome, block_on};

const HEADER: &str = "What the agent holding the floor is working on:";
const BRIEF: &str = "Fixing the flaky nightly build; retrying twice was just decided.";

proptest! {
    #[test]
    fn a_brief_follows_the_fixed_instructions_verbatim_under_its_header(
        brief in "\\PC{1,120}".prop_filter("not blank", |text| !text.trim().is_empty())
    ) {
        for taking_over in [false, true] {
            let fixed = instructions(taking_over, None);
            prop_assert_eq!(
                instructions(taking_over, Some(&brief)),
                format!("{fixed}\n\n{HEADER}\n{brief}")
            );
        }
    }

    #[test]
    fn a_blank_brief_leaves_the_instructions_unchanged(blank in "[ \t\r\n]{0,6}") {
        for taking_over in [false, true] {
            prop_assert_eq!(
                instructions(taking_over, Some(&blank)),
                instructions(taking_over, None)
            );
        }
    }
}

#[test]
fn the_initiation_message_carries_the_brief_in_its_prompt_override() {
    let message = initiation("v", false, Some(BRIEF));
    assert_eq!(
        message["conversation_config_override"]["agent"]["prompt"]["prompt"],
        json!(instructions(false, Some(BRIEF)))
    );
}

async fn connect_with(address: SocketAddr, brief: Option<&str>) -> Outcome {
    let url = format!("ws://{address}/");
    let request = SessionRequest {
        brief,
        ..request_for(&url, "v", false)
    };
    LiveClient::connect(&request).await?;
    Ok(())
}

async fn instructions_sent(brief: Option<&str>) -> Result<Value, Box<dyn Error>> {
    let started = json!({"type": "conversation_initiation_metadata"});
    let mut server = start(&[started]).await?;
    connect_with(server.address, brief).await?;
    let initiation = server.received.recv().await.unwrap_or_default();
    Ok(initiation["conversation_config_override"]["agent"]["prompt"]["prompt"].clone())
}

#[test]
fn the_handshake_sends_the_brief_to_the_live_provider() -> Outcome {
    let with_brief = block_on(instructions_sent(Some(BRIEF)))??;
    let without_brief = block_on(instructions_sent(None))??;
    assert_eq!(with_brief, json!(instructions(false, Some(BRIEF))));
    assert_eq!(without_brief, json!(instructions(false, None)));
    Ok(())
}
