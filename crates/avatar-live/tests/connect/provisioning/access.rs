use std::sync::Arc;

use avatar_live::client::provisioning::{AgentApi, signed_url};

use super::ledger::KEY;
use super::server::{Shared, start};
use super::{Failure, fresh, provisioned};
use crate::support::{Outcome, block_on};

async fn signed(ledger: &Shared) -> Result<String, Failure> {
    let address = start(Arc::clone(ledger)).await?;
    Ok(signed_url(
        &AgentApi::new(&format!("http://{address}"), KEY),
        "agents-1",
    )
    .await?)
}

#[test]
fn the_signed_url_is_read_from_the_api() -> Outcome {
    assert_eq!(block_on(signed(&fresh()))??, "wss://voice.example/agent");
    Ok(())
}

#[test]
fn a_refused_key_is_reported_with_the_status_and_never_echoed() -> Outcome {
    let failure = block_on(provisioned(&fresh(), "wrong-secret"))?
        .err()
        .ok_or("accepted")?;
    let message = failure.to_string();
    assert!(
        message.contains("401") && message.contains("convai_read"),
        "{message}"
    );
    assert!(!message.contains("wrong-secret"));
    Ok(())
}
