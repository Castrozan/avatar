use std::collections::BTreeMap;

use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct Host {
    pub(crate) listening: bool,
    pub(crate) meeting: bool,
}

#[derive(Deserialize)]
pub(crate) struct Request {
    pub(crate) path: String,
    pub(crate) headers: BTreeMap<String, String>,
}

#[derive(Deserialize)]
pub(crate) struct Expected {
    pub(crate) kind: String,
    pub(crate) status: Option<u16>,
    pub(crate) headers: Option<Vec<(String, String)>>,
    pub(crate) body_hex: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct Scenario {
    pub(crate) requests: Vec<Request>,
    pub(crate) attached: bool,
    pub(crate) host: Host,
    pub(crate) levels: bool,
    pub(crate) missing: Vec<String>,
    pub(crate) results: Vec<Expected>,
    pub(crate) levels_requested: usize,
    pub(crate) stderr: String,
}
