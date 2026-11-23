use super::super::support::TestResult;
use super::client::{ALLOWED_ORIGIN, assert_refused, connect, running_server};

#[tokio::test]
async fn server_listens_on_loopback_only() -> TestResult {
    let server = running_server().await?;
    assert_eq!(server.address.ip().to_string(), "127.0.0.1");
    Ok(())
}

#[tokio::test]
async fn allowed_origin_completes_the_handshake() -> TestResult {
    let server = running_server().await?;
    assert!(connect(server.address, Some(ALLOWED_ORIGIN)).await.is_ok());
    Ok(())
}

#[tokio::test]
async fn origin_on_another_port_is_refused_with_401() -> TestResult {
    assert_refused(Some("http://localhost:3001")).await
}

#[tokio::test]
async fn missing_origin_is_refused_with_401() -> TestResult {
    assert_refused(None).await
}

#[tokio::test]
async fn https_origin_is_refused_with_401() -> TestResult {
    assert_refused(Some("https://localhost:3000")).await
}

#[tokio::test]
async fn uppercase_host_origin_is_refused_with_401() -> TestResult {
    assert_refused(Some("http://LOCALHOST:3000")).await
}

#[tokio::test]
async fn origin_with_trailing_slash_is_refused_with_401() -> TestResult {
    assert_refused(Some("http://localhost:3000/")).await
}

#[tokio::test]
async fn origin_with_longer_port_is_refused_with_401() -> TestResult {
    assert_refused(Some("http://localhost:30001")).await
}

#[tokio::test]
async fn empty_origin_is_refused_with_401() -> TestResult {
    assert_refused(Some("")).await
}
