use std::fs::File;
use std::io;
use std::path::Path;

use super::Transport;
use crate::integrity::Integrity;
use crate::manifest::Download;
use crate::platform::files::create_parent;

pub(super) fn fetch_verified(
    download: &Download,
    integrity: &Integrity,
    part: &Path,
    transport: &dyn Transport,
) -> io::Result<()> {
    create_parent(part)?;
    let mut verifier = integrity.verifier(File::create(part)?);
    transport.download(&download.url, &mut verifier)?;
    let digest = verifier.finish();
    integrity
        .matches(&digest)
        .then_some(())
        .ok_or_else(|| mismatch(download, &digest))
}

fn mismatch(download: &Download, digest: &[u8]) -> io::Error {
    io::Error::other(format!(
        "{} failed verification: expected {}, received {}",
        download.name,
        download.integrity,
        Integrity::describe(digest)
    ))
}
