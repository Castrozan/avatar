use serde_json::json;

use super::socket::identified;
use super::stage::{change_relayed_from_the_renderer, request_character, spoken_voices, start};
use crate::TestResult;

#[tokio::test]
async fn a_character_requested_without_a_renderer_speaks_in_its_voice() -> TestResult {
    let stage = start().await?;
    let mut agent = identified(&stage.url, "agent").await?;
    let answer = request_character(&mut agent, "second").await?;
    let voices = spoken_voices(&stage, &mut agent).await?;
    stage.shutdown.shutdown();
    assert_eq!(answer["type"], "characterChanged");
    assert_eq!(voices, [json!("secondCloud")]);
    Ok(())
}

#[tokio::test]
async fn an_unknown_character_is_refused_and_the_voice_stays() -> TestResult {
    let stage = start().await?;
    let mut agent = identified(&stage.url, "agent").await?;
    let answer = request_character(&mut agent, "nobody").await?;
    let voices = spoken_voices(&stage, &mut agent).await?;
    stage.shutdown.shutdown();
    assert_eq!(answer["error"], "Unknown character: nobody");
    assert_eq!(voices, [json!("firstCloud")]);
    Ok(())
}

#[tokio::test]
async fn a_character_the_renderer_shows_speaks_in_its_voice() -> TestResult {
    let stage = start().await?;
    let mut agent = identified(&stage.url, "agent").await?;
    let relayed = change_relayed_from_the_renderer(&stage, &mut agent).await?;
    let voices = spoken_voices(&stage, &mut agent).await?;
    stage.shutdown.shutdown();
    assert_eq!(relayed["name"], "second");
    assert_eq!(voices, [json!("secondCloud")]);
    Ok(())
}
