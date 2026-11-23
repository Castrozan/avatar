use std::fs;
use std::io;
use std::path::Path;

pub(crate) fn create_parent(path: &Path) -> io::Result<()> {
    path.parent().map_or(Ok(()), fs::create_dir_all)
}
