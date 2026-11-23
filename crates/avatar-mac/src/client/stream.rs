use super::STREAM_ATTACH_WAIT_SECONDS;
use super::endpoint::{SeatEndpoint, StreamConnection};
use crate::failure::Failure;

#[must_use]
pub fn stream_header(rate: u32) -> String {
    format!("{{\"sampleRate\": {rate}}}")
}

pub fn open_stream_connection<Endpoint: SeatEndpoint>(
    endpoint: &mut Endpoint,
    rate: u32,
    no_page_message: &str,
) -> Result<Endpoint::Connection, Failure> {
    endpoint.wait_until_attached(STREAM_ATTACH_WAIT_SECONDS, no_page_message)?;
    let mut connection = endpoint.connect_stream()?;
    connection.send(stream_header(rate).as_bytes())?;
    Ok(connection)
}

fn ensure_open<Endpoint: SeatEndpoint>(
    current: Option<Endpoint::Connection>,
    endpoint: &mut Endpoint,
    settings: (u32, &str),
) -> Result<Option<Endpoint::Connection>, Failure> {
    if current.is_some() {
        return Ok(current);
    }
    open_stream_connection(endpoint, settings.0, settings.1).map(Some)
}

fn send_on<Connection: StreamConnection>(
    connection: Option<&mut Connection>,
    pcm: &[u8],
) -> Result<(), Failure> {
    connection.map_or(Ok(()), |open| open.send(pcm))
}

pub struct SeatStream<Endpoint: SeatEndpoint> {
    rate: u32,
    no_page_message: String,
    connection: Option<Endpoint::Connection>,
}

impl<Endpoint: SeatEndpoint> SeatStream<Endpoint> {
    #[must_use]
    pub fn new(rate: u32, no_page_message: &str) -> Self {
        Self {
            rate,
            no_page_message: no_page_message.to_owned(),
            connection: None,
        }
    }

    pub fn open(&mut self, endpoint: &mut Endpoint) -> Result<(), Failure> {
        let settings = (self.rate, self.no_page_message.as_str());
        self.connection = ensure_open(self.connection.take(), endpoint, settings)?;
        Ok(())
    }

    pub fn send(&mut self, endpoint: &mut Endpoint, pcm: &[u8]) -> Result<(), Failure> {
        self.open(endpoint)?;
        send_on(self.connection.as_mut(), pcm)
    }

    pub fn clear(&self, endpoint: &mut Endpoint) {
        endpoint.hush();
    }

    pub fn close(&mut self) {
        self.connection.iter_mut().for_each(StreamConnection::close);
    }
}
