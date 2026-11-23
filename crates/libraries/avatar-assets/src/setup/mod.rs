mod archive;
mod fetch;
mod install;

use std::io::{self, Write};

use crate::manifest::Download;
use crate::platform::layout::DataDirectory;
use crate::prepare;

pub use install::install_one;

pub trait Transport {
    fn download(&self, url: &str, sink: &mut dyn Write) -> io::Result<()>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Installed,
    AlreadyPresent,
}

pub fn install_all(
    downloads: &[Download],
    data: &DataDirectory,
    transport: &dyn Transport,
    report: &mut dyn FnMut(&Download, Outcome),
) -> io::Result<()> {
    for download in downloads {
        report(download, install_one(download, data, transport)?);
    }
    prepare::renderer(data).map(drop)
}

#[cfg(test)]
mod tests;
