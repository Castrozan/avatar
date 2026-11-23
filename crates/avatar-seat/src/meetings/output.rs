use std::io::Write;

pub trait Output {
    fn line(&self, text: &str);
}

pub struct StandardOutput;

impl Output for StandardOutput {
    fn line(&self, text: &str) {
        let mut stdout = std::io::stdout().lock();
        stdout
            .write_all(format!("{text}\n").as_bytes())
            .unwrap_or_default();
    }
}
