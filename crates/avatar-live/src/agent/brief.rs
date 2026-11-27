use avatar_floor::addressee::Addressee;

use super::report::AgentQuestion;

pub const ANSWER_TAG: &str = "LIVE-ANSWER";
pub const SENDER: &str = "avatar live voice";
pub const STILL_WORKING: &str = "The agent is still working on that and has not answered yet.";
pub const NO_ANSWER_IN_TIME: &str =
    "The agent gave no answer within 5 minutes; it may still be busy, so ask again in a while.";
pub const NO_ANSWER_REASON: &str = "no answer within 5 minutes";
pub const STOPPED_REASON: &str = "the live voice stopped";

const NOTICE: &str = "Your answer to {tag} never reached Lucas ({reason}); answer it again now with a closing line {tag}: <one sentence>.";

const BRIEF: &str = "[avatar-live] Lucas asked this by voice in the open live avatar conversation, and the live voice will read your answer aloud to him: {request} -- Do the work, then begin your closing message with {tag} and a colon, followed by one short plain spoken sentence that answers him, in the language he used, with no markdown, code or file paths. Answer that way at once even while background work of yours still runs.";

#[must_use]
pub fn brief(question: &AgentQuestion) -> String {
    BRIEF
        .replace("{tag}", &question.answer_tag())
        .replace("{request}", &question.request)
}

#[must_use]
pub fn send_arguments(holder: &Addressee, question: &AgentQuestion) -> Vec<String> {
    ["send", "--sender", SENDER, &holder.pane, &brief(question)]
        .map(str::to_owned)
        .to_vec()
}

#[must_use]
pub fn status_arguments(holder: &Addressee, task_id: &str) -> Vec<String> {
    ["status", &holder.pane, task_id]
        .map(str::to_owned)
        .to_vec()
}

#[must_use]
pub fn could_not_finish(reason: &str) -> String {
    format!("The agent could not finish that: {reason}")
}

#[must_use]
pub fn notice_arguments(holder: &Addressee, question: &AgentQuestion, reason: &str) -> Vec<String> {
    let notice = NOTICE
        .replace("{tag}", &question.answer_tag())
        .replace("{reason}", reason);
    ["notify", "--sender", SENDER, &holder.pane, &notice]
        .map(str::to_owned)
        .to_vec()
}
