use std::io;
use std::path::PathBuf;

use crate::page::PageFiles;

pub struct DirectoryFiles(pub PathBuf);

impl PageFiles for DirectoryFiles {
    fn read(&self, file: &str) -> io::Result<Vec<u8>> {
        std::fs::read(self.0.join(file))
    }
}
