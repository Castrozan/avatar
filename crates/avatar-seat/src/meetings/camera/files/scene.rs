use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct Point {
    #[serde(rename = "x")]
    horizontal: f64,
    #[serde(rename = "y")]
    vertical: f64,
}

impl Point {
    fn new(horizontal: f64, vertical: f64) -> Self {
        Self {
            horizontal,
            vertical,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct Header {
    id: &'static str,
    versioned_id: &'static str,
    name: &'static str,
    enabled: bool,
}

impl Header {
    pub(crate) fn new(id: &'static str, name: &'static str) -> Self {
        Self {
            id,
            versioned_id: id,
            name,
            enabled: true,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct BrowserSettings {
    pub(crate) url: String,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) fps: u32,
    pub(crate) fps_custom: bool,
}

#[derive(Serialize)]
pub(crate) struct BrowserSource {
    #[serde(flatten)]
    pub(crate) header: Header,
    pub(crate) muted: bool,
    pub(crate) volume: f64,
    pub(crate) settings: BrowserSettings,
}

#[derive(Serialize)]
pub(crate) struct Placement {
    pos: Point,
    scale: Point,
    rot: f64,
    bounds_type: u32,
}

impl Placement {
    pub(crate) fn identity() -> Self {
        Self {
            pos: Point::new(0.0, 0.0),
            scale: Point::new(1.0, 1.0),
            rot: 0.0,
            bounds_type: 0,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct SceneItem {
    pub(crate) id: u32,
    pub(crate) name: &'static str,
    pub(crate) visible: bool,
    pub(crate) locked: bool,
    #[serde(flatten)]
    pub(crate) placement: Placement,
}

#[derive(Serialize)]
pub(crate) struct SceneSettings {
    pub(crate) id_counter: u32,
    pub(crate) items: Vec<SceneItem>,
}

#[derive(Serialize)]
pub(crate) struct Scene {
    #[serde(flatten)]
    pub(crate) header: Header,
    pub(crate) settings: SceneSettings,
}

#[derive(Serialize)]
#[serde(untagged)]
pub(crate) enum Source {
    Browser(BrowserSource),
    Scene(Scene),
}
