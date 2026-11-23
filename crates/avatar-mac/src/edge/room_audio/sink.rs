use crate::client::{SeatEndpoint, SeatStream};
use crate::failure::Failure;

pub trait SeatSink: Send {
    fn open(&mut self) -> Result<(), Failure>;
    fn send(&mut self, pcm: &[u8]) -> Result<(), Failure>;
    fn close(&mut self);
}

pub struct SeatStreamSink<Endpoint: SeatEndpoint> {
    endpoint: Endpoint,
    stream: SeatStream<Endpoint>,
}

impl<Endpoint: SeatEndpoint> SeatStreamSink<Endpoint> {
    #[must_use]
    pub fn new(endpoint: Endpoint, rate: u32, no_page_message: &str) -> Self {
        Self {
            endpoint,
            stream: SeatStream::new(rate, no_page_message),
        }
    }
}

impl<Endpoint: SeatEndpoint + Send> SeatSink for SeatStreamSink<Endpoint>
where
    Endpoint::Connection: Send,
{
    fn open(&mut self) -> Result<(), Failure> {
        self.stream.open(&mut self.endpoint)
    }

    fn send(&mut self, pcm: &[u8]) -> Result<(), Failure> {
        self.stream.send(&mut self.endpoint, pcm)
    }

    fn close(&mut self) {
        self.stream.close();
    }
}
