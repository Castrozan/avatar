use axum::Router;
use axum::routing::get;
use serde_json::{Value, json};
use tokio::net::TcpListener;

use crate::run::{endpoints, run};
use crate::script::only_identifying;
use crate::server::serve;
use crate::support::{TestResult, check};

pub(crate) async fn seat_server(body: &'static str) -> Result<String, Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    let router = Router::new().route("/health", get(move || async move { body }));
    tokio::spawn(async move { drop(axum::serve(listener, router).await) });
    Ok(format!("http://{address}/health"))
}

pub(crate) async fn seat_url(
    body: Option<&'static str>,
    unreachable: String,
) -> Result<String, Box<dyn std::error::Error>> {
    match body {
        Some(text) => seat_server(text).await,
        None => Ok(unreachable),
    }
}

pub(crate) async fn spoken_output(
    seat_body: Option<&'static str>,
) -> Result<Value, Box<dyn std::error::Error>> {
    let server = serve(only_identifying).await?;
    let mut settings = endpoints(&server);
    settings.seat_health_url = seat_url(seat_body, settings.seat_health_url).await?;
    run(&["speak", "hello"], &settings).await;
    Ok(server.messages()[1].clone())
}

#[tokio::test]
pub(crate) async fn speech_goes_to_the_seat_when_a_mac_holds_it() -> TestResult {
    let attached = spoken_output(Some(r#"{"attached": true}"#)).await?;
    let detached = spoken_output(Some(r#"{"attached": false}"#)).await?;
    let unreachable = spoken_output(None).await?;
    let garbled = spoken_output(Some("not json")).await?;
    check(attached["output"] == "remote", &attached)?;
    let others = [detached, unreachable, garbled];
    check(
        others.iter().all(|command| command["output"] == "both"),
        json!(others),
    )
}
