pub const SPEECH: &str = "Every utterance is a speech-to-text transcript of the person, so names and words may be misheard.";

pub const QUESTION_INSTRUCTIONS: &str = "Whom does the person speak to in their latest utterance, the last message, expecting that agent to answer or act right now? An agent counts only when the person calls it by name as the one they are talking to. Hearing a name does not make its agent the addressee: mentioning or talking about an agent, asking someone to tell, ask, contact or write to an agent, doing something for or about an agent, quoting or singing its name, or repeating the name while discussing names all leave that agent out.";

pub const NOBODY_DESCRIPTION: &str = "The speech is not directed at any agent: singing, thinking aloud, reading aloud, or talking to another person.";

#[must_use]
pub fn developer_context(holder: Option<&str>, listed_candidates: &str) -> String {
    match holder {
        None => format!(
            "A room microphone hears a person near voice-activated AI agents named {listed_candidates}. Nobody is in a conversation with them yet; an agent wakes only when the person calls it to speak with it. {SPEECH}"
        ),
        Some(holder) => format!(
            "A person is talking by voice with an AI agent named {holder}. Other live AI agents named {listed_candidates} can take over the conversation when the person turns to address them. {SPEECH}"
        ),
    }
}

#[must_use]
pub fn holder_description(holder: &str) -> String {
    format!(
        "The person keeps talking to {holder}, the agent already in the conversation, including when other agents are only mentioned, talked about, or to be contacted through {holder}."
    )
}

#[must_use]
pub fn candidate_description(name: &str) -> String {
    format!(
        "The person calls {name} by name to talk to {name} now, so {name} should answer; not when {name} is only the topic, someone to be told, asked or written to, or a name being sung or discussed."
    )
}
