use super::layout::{COLLECTION_NAME, CameraConfig};
use super::obs::ObsCamera;
use super::start_script::START_VIRTUAL_CAMERA;
use super::{profile_settings, scene_collection_text};
use crate::edges::EdgeFailure;

fn documents(config: &CameraConfig) -> Vec<(String, String)> {
    let collection = format!("{}/{COLLECTION_NAME}.json", config.scenes_directory());
    vec![
        (collection, scene_collection_text(config)),
        (
            format!("{}/basic.ini", config.profile_directory()),
            profile_settings(),
        ),
        (config.start_script.clone(), START_VIRTUAL_CAMERA.to_owned()),
    ]
}

impl ObsCamera<'_> {
    fn make_directories(&self, directories: &[String]) -> Result<(), EdgeFailure> {
        directories
            .iter()
            .try_for_each(|directory| self.store.make_directory(directory))
    }

    pub fn write_configuration(&self) -> Result<(), EdgeFailure> {
        let config = self.config;
        let (scenes, profile) = (config.scenes_directory(), config.profile_directory());
        self.make_directories(&[scenes, profile, config.report_directory()])?;
        let write = |(path, text): &(String, String)| self.store.write_text(path, text);
        documents(config).iter().try_for_each(write)
    }
}
