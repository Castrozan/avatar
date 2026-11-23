use super::layout::COLLECTION_NAME;
use super::obs::ObsCamera;
use super::{profile_settings, scene_collection_text};
use crate::edges::EdgeFailure;

impl ObsCamera<'_> {
    fn make_directories(&self, directories: &[String]) -> Result<(), EdgeFailure> {
        directories
            .iter()
            .try_for_each(|directory| self.store.make_directory(directory))
    }

    pub fn write_configuration(&self) -> Result<(), EdgeFailure> {
        let config = self.config;
        let scenes = config.scenes_directory();
        let profile = config.profile_directory();
        let directories = [scenes.clone(), profile.clone(), config.report_directory()];
        self.make_directories(&directories)?;
        let collection = format!("{scenes}/{COLLECTION_NAME}.json");
        self.store
            .write_text(&collection, &scene_collection_text(config))?;
        let settings = format!("{profile}/basic.ini");
        self.store.write_text(&settings, &profile_settings())
    }
}
