mod cast;
mod embedded;
mod integrity;
mod manifest;
mod platform;
mod prepare;
mod resolve;
mod setup;

pub use manifest::{Download, embedded_downloads, parse_downloads};
pub use platform::environment::{Environment, ProcessEnvironment};
pub use platform::layout::DataDirectory;
pub use resolve::{Asset, ordinary_word_paths, resolve, resolve_in};
pub use setup::{Outcome, Transport, install_all, install_one};

#[cfg(test)]
mod tests;
