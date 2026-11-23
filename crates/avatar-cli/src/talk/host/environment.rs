use avatar_conversation::talk::ports::Environment;
use nix::unistd::gethostname;

pub(crate) struct ProcessEnvironment;

impl Environment for ProcessEnvironment {
    fn variable(&self, name: &str) -> Option<String> {
        std::env::var(name).ok()
    }

    fn hostname(&self) -> String {
        gethostname()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default()
    }
}
