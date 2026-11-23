#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Frame {
    Binary(Vec<u8>),
    Text(String),
}

#[must_use]
pub fn whole_samples(pcm: &[u8]) -> &[u8] {
    &pcm[..pcm.len() - pcm.len() % 2]
}
