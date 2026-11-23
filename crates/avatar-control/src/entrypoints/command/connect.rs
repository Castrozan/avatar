use std::io::ErrorKind;
use std::time::Duration;

use tokio::net::TcpStream;
use tokio::time::{Instant, sleep};
use tokio_tungstenite::tungstenite::Error;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

pub type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

fn refused(error: &Error) -> bool {
    matches!(error, Error::Io(io) if io.kind() == ErrorKind::ConnectionRefused)
}

fn wants_retry<Connected>(outcome: &Result<Connected, Error>, give_up_at: Instant) -> bool {
    outcome.as_ref().err().is_some_and(refused) && Instant::now() < give_up_at
}

pub async fn connect_to_starting_server(
    url: &str,
    patience: Duration,
    retry: Duration,
) -> Result<Socket, Error> {
    let give_up_at = Instant::now() + patience;
    let mut outcome = connect_async(url).await;
    while wants_retry(&outcome, give_up_at) {
        sleep(retry).await;
        outcome = connect_async(url).await;
    }
    outcome.map(|(socket, _response)| socket)
}
