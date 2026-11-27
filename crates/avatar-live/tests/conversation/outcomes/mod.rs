mod answers;
mod failures;
mod stop;
mod transport;

use std::future::{Future, pending};

use avatar_live::agent::route::AgentRoute;
use avatar_live::conversation::{HandOffDesk, Podium};
use avatar_live::events::TaskRequest;
use serde_json::Value;

use self::transport::{Replies, ScriptedTransport};
use super::fakes::{Recorder, Screen};
use super::scripted::{Holder, Voice};
use crate::support::{VirtualClock, addressee, block_on};

type Desk = HandOffDesk<Holder, AgentRoute<ScriptedTransport>, Voice, Screen>;

const OUTCOME_MARKERS: [&str; 3] = [" answered by ", " failed for ", " unanswered for "];

struct Run {
    lines: Vec<String>,
    spoken: Vec<Value>,
    commands: Vec<Vec<String>>,
}

impl Run {
    fn task_lines(&self) -> Vec<&str> {
        let lines = self.lines.iter().map(String::as_str);
        lines.filter(|line| line.starts_with("task ")).collect()
    }

    fn task_line(&self, index: usize) -> &str {
        self.task_lines().get(index).copied().unwrap_or_default()
    }

    fn outcome_count(&self) -> usize {
        let is_outcome = |line: &str| OUTCOME_MARKERS.iter().any(|marker| line.contains(marker));
        self.task_lines()
            .into_iter()
            .filter(|line| is_outcome(line))
            .count()
    }

    fn verbs(&self) -> Vec<&str> {
        let first_words = self.commands.iter().filter_map(|words| words.first());
        first_words.map(String::as_str).collect()
    }
}

fn desk_for(transport: ScriptedTransport, answered_before: u32, recorder: &Recorder) -> Desk {
    HandOffDesk {
        holders: Holder(Some(addressee("wW:pQZ", "Suzuka"))),
        route: AgentRoute::new(transport),
        podium: Podium {
            voice: Voice(Vec::new()),
            screen: Screen(recorder.clone()),
        },
        answer_number: answered_before,
    }
}

fn the_task() -> TaskRequest {
    TaskRequest {
        task_id: "t1".to_owned(),
        request: "check the build".to_owned(),
    }
}

fn collected(recorder: &Recorder, mut desk: Desk) -> Run {
    Run {
        lines: recorder.printed.borrow().clone(),
        spoken: std::mem::take(&mut desk.podium.voice.0),
        commands: desk.route.into_transport().commands,
    }
}

fn drive(
    transport: ScriptedTransport,
    answered_before: u32,
    stopped: impl Future<Output = ()>,
) -> Result<Run, std::io::Error> {
    let recorder = Recorder::default();
    let mut desk = desk_for(transport, answered_before, &recorder);
    block_on(desk.hand_off(&the_task(), &mut VirtualClock::default(), stopped))?;
    Ok(collected(&recorder, desk))
}

fn run_after(answered_before: u32, replies: Replies) -> Result<Run, std::io::Error> {
    drive(ScriptedTransport::new(replies), answered_before, pending())
}

fn run(replies: Replies) -> Result<Run, std::io::Error> {
    run_after(12, replies)
}

fn run_stopped(replies: Replies) -> Result<Run, std::io::Error> {
    let (transport, stop) = ScriptedTransport::stopping_at_the_first_status(replies);
    drive(transport, 12, async move {
        let _ = stop.await;
    })
}

fn notice(number: u32, reason: &str) -> Vec<String> {
    let tag = format!("LIVE-ANSWER-{number}");
    let text = format!(
        "Your answer to {tag} never reached Lucas ({reason}); answer it again now with a closing line {tag}: <one sentence>."
    );
    ["notify", "--sender", "avatar live voice", "wW:pQZ", &text]
        .map(str::to_owned)
        .to_vec()
}
