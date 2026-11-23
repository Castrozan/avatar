mod configuration;
mod files;
mod filesystem;
mod launch;
mod layout;
mod obs;
mod report;
mod start_script;

use crate::edges::EdgeFailure;

pub use files::{profile_settings, python_json, scene_collection_text};
pub use filesystem::FileSystemStore;
pub use layout::CameraConfig;
pub use obs::ObsCamera;

pub trait CameraStore {
    fn exists(&self, path: &str) -> bool;
    fn make_directory(&self, path: &str) -> Result<(), EdgeFailure>;
    fn write_text(&self, path: &str, text: &str) -> Result<(), EdgeFailure>;
    fn read_text(&self, path: &str) -> Option<String>;
    fn remove(&self, path: &str) -> Result<(), EdgeFailure>;
}
