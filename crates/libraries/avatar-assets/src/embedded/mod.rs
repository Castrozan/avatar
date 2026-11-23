use std::fs;
use std::io;
use std::path::Path;

use include_dir::{Dir, include_dir};

use crate::platform::files::create_parent;

pub(crate) static RENDERER: Dir<'static> =
    include_dir!("$CARGO_MANIFEST_DIR/../../../runtime/renderer");
pub(crate) static SEAT_PAGE: Dir<'static> =
    include_dir!("$CARGO_MANIFEST_DIR/../../../runtime/scripts/conversation/remote_seat/page");
pub(crate) static WAKE_PAGE: Dir<'static> =
    include_dir!("$CARGO_MANIFEST_DIR/../../../runtime/scripts/wake/page");
pub(crate) const SERVANT_CHARACTERS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../runtime/cast/servant-characters.txt"
));
pub(crate) const DEFAULT_PORTRAIT: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../skills/avatar/avatars/jarvis.jpeg"
));

pub(crate) fn write_directory(source: &Dir<'_>, target: &Path) -> io::Result<()> {
    for file in source.files() {
        write_if_changed(&target.join(file.path()), file.contents())?;
    }
    source
        .dirs()
        .try_for_each(|directory| write_directory(directory, target))
}

pub(crate) fn write_if_changed(path: &Path, contents: &[u8]) -> io::Result<()> {
    if fs::read(path).is_ok_and(|existing| existing == contents) {
        return Ok(());
    }
    create_parent(path)?;
    let scratch = path.with_extension(format!("{}.partial", std::process::id()));
    fs::write(&scratch, contents)?;
    fs::rename(scratch, path)
}

#[cfg(test)]
mod tests;
