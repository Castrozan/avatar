use std::io;

use crate::host::state::HostSnapshot;

pub trait PageFiles {
    fn read(&self, file: &str) -> io::Result<Vec<u8>>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageRequest {
    pub path_and_query: String,
    pub fetch_site: Option<String>,
}

pub struct PageState<'state> {
    pub attached: bool,
    pub host: HostSnapshot,
    pub files: &'state dyn PageFiles,
}

impl<'state> PageState<'state> {
    #[must_use]
    pub fn new(attached: bool, host: HostSnapshot, files: &'state dyn PageFiles) -> Self {
        Self {
            attached,
            host,
            files,
        }
    }
}
