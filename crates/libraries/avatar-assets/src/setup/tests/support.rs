use std::cell::Cell;
use std::collections::HashMap;
use std::io::{self, Write};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use flate2::Compression;
use flate2::write::GzEncoder;
use sha2::{Digest, Sha256};

use crate::manifest::{ArchiveKind, Download};
use crate::setup::Transport;

pub(super) type TestResult = Result<(), Box<dyn std::error::Error>>;

pub(super) struct FakeTransport {
    pub(super) bodies: HashMap<String, Vec<u8>>,
    pub(super) requests: Cell<usize>,
}

impl FakeTransport {
    pub(super) fn serving(url: &str, body: Vec<u8>) -> Self {
        Self {
            bodies: HashMap::from([(url.to_owned(), body)]),
            requests: Cell::new(0),
        }
    }
}

impl Transport for FakeTransport {
    fn download(&self, url: &str, sink: &mut dyn Write) -> io::Result<()> {
        self.requests.set(self.requests.get() + 1);
        let body = self
            .bodies
            .get(url)
            .ok_or_else(|| io::Error::other("404"))?;
        sink.write_all(body)
    }
}

pub(super) fn integrity_of(content: &[u8]) -> String {
    format!("sha256-{}", STANDARD.encode(Sha256::digest(content)))
}

pub(super) fn download(destination: &str, content: &[u8]) -> Download {
    Download {
        name: "sample".to_owned(),
        url: "http://127.0.0.1/sample".to_owned(),
        integrity: integrity_of(content),
        destination: destination.to_owned(),
        archive: None,
    }
}

pub(super) fn archive_download(destination: &str, content: &[u8]) -> Download {
    Download {
        archive: Some(ArchiveKind::TarGzip),
        ..download(destination, content)
    }
}

pub(super) fn tarball(files: &[(&str, &str)]) -> io::Result<Vec<u8>> {
    let mut builder = tar::Builder::new(GzEncoder::new(Vec::new(), Compression::default()));
    for (path, body) in files {
        let mut header = tar::Header::new_gnu();
        header.set_size(body.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder.append_data(&mut header, path, body.as_bytes())?;
    }
    builder.into_inner()?.finish()
}

pub(super) fn empty_data() -> io::Result<(tempfile::TempDir, crate::platform::layout::DataDirectory)>
{
    let directory = tempfile::tempdir()?;
    let data = crate::platform::layout::DataDirectory::at(directory.path().to_path_buf());
    Ok((directory, data))
}

pub(super) fn tarball_with_raw_path(path: &str, body: &str) -> io::Result<Vec<u8>> {
    let mut header = tar::Header::new_gnu();
    header.as_old_mut().name[..path.len()].copy_from_slice(path.as_bytes());
    header.set_size(body.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    let mut builder = tar::Builder::new(GzEncoder::new(Vec::new(), Compression::default()));
    builder.append(&header, body.as_bytes())?;
    builder.into_inner()?.finish()
}
