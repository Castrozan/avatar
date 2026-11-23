use std::future::Future;
use std::io;

pub trait Playback: Send {
    fn write(&mut self, bytes: &[u8]) -> impl Future<Output = io::Result<()>> + Send;
    fn end(&mut self) -> impl Future<Output = ()> + Send;
}

pub trait Capture: Send {
    fn read(&mut self, limit: usize) -> impl Future<Output = io::Result<Vec<u8>>> + Send;
    fn end(&mut self) -> impl Future<Output = ()> + Send;
}

pub trait AudioTools: Send + Sync + 'static {
    type Playback: Playback;
    type Capture: Capture;

    fn playback(
        &self,
        command: Vec<String>,
    ) -> impl Future<Output = io::Result<Self::Playback>> + Send;
    fn capture(
        &self,
        command: Vec<String>,
    ) -> impl Future<Output = io::Result<Self::Capture>> + Send;
}
