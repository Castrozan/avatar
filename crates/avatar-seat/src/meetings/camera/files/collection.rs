use super::super::layout::{
    COLLECTION_NAME, CameraConfig, FRAME_HEIGHT, FRAME_WIDTH, FRAMES_PER_SECOND, SCENE_NAME,
    SOURCE_NAME, avatar_view_url,
};
use super::document::{
    Modules, SceneCollection, SceneName, Selection, StartScript, StartScriptSettings, Unused,
};
use super::python_json::python_json;
use super::scene::{
    BrowserSettings, BrowserSource, Header, Placement, Scene, SceneItem, SceneSettings, Source,
};

fn browser_settings() -> BrowserSettings {
    let (width, height, fps) = (FRAME_WIDTH, FRAME_HEIGHT, FRAMES_PER_SECOND);
    BrowserSettings {
        url: avatar_view_url(),
        width,
        height,
        fps,
        fps_custom: true,
    }
}

fn browser_source() -> Source {
    let header = Header::new("browser_source", SOURCE_NAME);
    Source::Browser(BrowserSource {
        header,
        muted: false,
        volume: 1.0,
        settings: browser_settings(),
    })
}

fn scene_item() -> SceneItem {
    let placement = Placement::identity();
    SceneItem {
        id: 1,
        name: SOURCE_NAME,
        visible: true,
        locked: false,
        placement,
    }
}

fn scene() -> Source {
    let settings = SceneSettings {
        id_counter: 1,
        items: vec![scene_item()],
    };
    Source::Scene(Scene {
        header: Header::new("scene", SCENE_NAME),
        settings,
    })
}

fn modules(config: &CameraConfig) -> Modules {
    let settings = StartScriptSettings {
        report_path: config.report_path(),
    };
    let script = StartScript {
        path: config.start_script.clone(),
        settings,
    };
    Modules {
        scripts_tool: vec![script],
    }
}

fn selection() -> Selection {
    Selection {
        name: COLLECTION_NAME,
        current_scene: SCENE_NAME,
        current_program_scene: SCENE_NAME,
    }
}

fn collection(config: &CameraConfig) -> SceneCollection {
    SceneCollection {
        selection: selection(),
        scene_order: vec![SceneName { name: SCENE_NAME }],
        sources: vec![browser_source(), scene()],
        unused: Unused::default(),
        modules: modules(config),
    }
}

#[must_use]
pub fn scene_collection_text(config: &CameraConfig) -> String {
    python_json(&collection(config))
}
