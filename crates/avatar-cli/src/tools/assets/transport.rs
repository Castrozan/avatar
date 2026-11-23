use std::io::{self, Write};

use avatar_assets::Transport;
use reqwest::blocking::{Client, Response};

pub(super) struct HttpTransport;

impl Transport for HttpTransport {
    fn download(&self, url: &str, sink: &mut dyn Write) -> io::Result<()> {
        let client = Client::builder()
            .timeout(None)
            .build()
            .map_err(io::Error::other)?;
        let mut response = client
            .get(url)
            .send()
            .and_then(Response::error_for_status)
            .map_err(io::Error::other)?;
        response.copy_to(sink).map(drop).map_err(io::Error::other)
    }
}
