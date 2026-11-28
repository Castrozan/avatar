use avatar_transcript::echo::is_python_space;
use avatar_transcript::json::Json;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;

use super::reading::{event_id, nested, required_text};
use super::{EventTranslator, MalformedEvent, RECENT_USER_CHARACTERS, Translation};
use crate::events::{Audio, LiveEvent, TaskRequest};
use crate::ordered_json::text;
use crate::session::{HAND_OFF_TOOL, HANDED_OFF, tool_result};

fn last_characters(content: &str, count: usize) -> String {
    let skipped = content.chars().count().saturating_sub(count);
    content.chars().skip(skipped).collect()
}

fn decoded(event: &Json) -> Result<Vec<u8>, MalformedEvent> {
    let encoded = required_text(event, "audio_event", "audio_base_64")?;
    let pcm = STANDARD.decode(encoded);
    pcm.map_err(|failure| MalformedEvent(failure.to_string()))
}

fn unknown_tool(tool_call_id: &str, name: Option<&str>) -> Translation {
    let reason = format!("unknown tool {}", name.unwrap_or_default());
    Translation {
        events: Vec::new(),
        replies: vec![tool_result(tool_call_id, &reason, true)],
    }
}

impl EventTranslator {
    pub(super) fn audio(&mut self, event: &Json) -> Result<Vec<LiveEvent>, MalformedEvent> {
        let id = event_id(event, "audio_event")?;
        let pcm = decoded(event)?;
        let stale = self
            .interrupted_at
            .is_some_and(|interrupted| id < interrupted);
        Ok(Some(LiveEvent::Audio(Audio { pcm }))
            .filter(|_| !stale)
            .into_iter()
            .collect())
    }

    pub(super) fn interruption(&mut self, event: &Json) -> Vec<LiveEvent> {
        let id = event_id(event, "interruption_event").ok();
        self.interrupted_at = id.or(self.interrupted_at);
        vec![LiveEvent::Interrupted]
    }

    pub(super) fn user_transcript(
        &mut self,
        event: &Json,
    ) -> Result<Vec<LiveEvent>, MalformedEvent> {
        let words = required_text(event, "user_transcription_event", "user_transcript")?;
        self.user_since_last_task.push_str(words);
        let recent = format!("{}{words}", self.recent_user);
        self.recent_user = last_characters(&recent, RECENT_USER_CHARACTERS);
        Ok(vec![LiveEvent::UserSaid(words.to_owned())])
    }

    pub(super) fn tool_call(&mut self, event: &Json) -> Result<Translation, MalformedEvent> {
        let tool_call_id = required_text(event, "client_tool_call", "tool_call_id")?;
        let name = text(nested(event, "client_tool_call", "tool_name"));
        Ok(match name {
            Some(HAND_OFF_TOOL) => self.hand_off(tool_call_id),
            other => unknown_tool(tool_call_id, other),
        })
    }

    fn hand_off(&mut self, tool_call_id: &str) -> Translation {
        let task = TaskRequest {
            task_id: tool_call_id.to_owned(),
            request: self.task_request(),
        };
        Translation {
            events: vec![LiveEvent::TaskRequested(task)],
            replies: vec![tool_result(tool_call_id, HANDED_OFF, false)],
        }
    }

    fn task_request(&mut self) -> String {
        let since = std::mem::take(&mut self.user_since_last_task);
        let chosen = [since.as_str(), self.recent_user.as_str()]
            .map(|candidate| candidate.trim_matches(is_python_space))
            .into_iter()
            .find(|candidate| !candidate.is_empty())
            .unwrap_or("");
        last_characters(chosen, RECENT_USER_CHARACTERS)
    }
}
