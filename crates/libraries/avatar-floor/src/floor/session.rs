#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Session {
    pub servant: String,
    pub pane: String,
    pub host: Option<String>,
}
