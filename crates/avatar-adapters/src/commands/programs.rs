use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default)]
pub struct Programs {
    directory: Option<PathBuf>,
}

impl Programs {
    #[must_use]
    pub const fn on_path() -> Self {
        Self { directory: None }
    }

    #[must_use]
    pub fn in_directory(directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: Some(directory.into()),
        }
    }

    #[must_use]
    pub fn resolve(&self, mut command: Vec<String>) -> Vec<String> {
        relocate(self.directory.as_deref(), &mut command);
        command
    }

    #[must_use]
    pub fn words(&self, words: &[&str]) -> Vec<String> {
        self.resolve(words.iter().map(|word| (*word).to_owned()).collect())
    }
}

fn located(directory: Option<&Path>, program: &str) -> String {
    directory.map_or_else(
        || program.to_owned(),
        |directory| directory.join(program).to_string_lossy().into_owned(),
    )
}

fn relocate(directory: Option<&Path>, command: &mut [String]) {
    if let Some(program) = command.first_mut() {
        *program = located(directory, program);
    }
}
