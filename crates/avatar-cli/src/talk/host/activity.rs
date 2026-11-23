use avatar_conversation::talk::ports::ActivityLog;
use serde_json::{Map, Value, json};

use crate::support::wake_log::{report_to_log, stamped};

pub(crate) struct LoggedActivity;

fn report(phase: &str, details: Map<String, Value>) {
    report_to_log(&stamped(phase, details));
}

impl ActivityLog for LoggedActivity {
    fn report_talk(&mut self, action: &str, pane: Option<&str>) {
        let mut details = Map::new();
        details.insert("action".to_owned(), json!(action));
        details.insert("pane".to_owned(), json!(pane));
        report("talk", details);
    }

    fn report_speech(&mut self, pane: Option<&str>, details: &Map<String, Value>) {
        let mut record = Map::new();
        record.insert("pane".to_owned(), json!(pane));
        record.extend(details.clone());
        report("speech", record);
    }
}
