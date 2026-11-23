use avatar_holder::floor_holder::{ReplySource, Reporter, Speech, SpeechFailure};
use avatar_holder::hermes_turn::HermesTurnError;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

#[derive(Default)]
pub(crate) struct VoiceState {
    pub(crate) replies: VecDeque<Result<String, HermesTurnError>>,
    pub(crate) speak_failure: Option<String>,
    pub(crate) spoken: Vec<String>,
    pub(crate) reports: Vec<String>,
}

#[derive(Clone, Default)]
pub(crate) struct SharedVoice(pub(crate) Rc<RefCell<VoiceState>>);

impl ReplySource for SharedVoice {
    fn reply_to(&mut self, _heard: &str) -> Result<String, HermesTurnError> {
        let next = self.0.borrow_mut().replies.pop_front();
        next.unwrap_or_else(|| Err(HermesTurnError::new("unscripted reply")))
    }
}

impl Speech for SharedVoice {
    fn speak(&mut self, text: &str) -> Result<(), SpeechFailure> {
        let mut state = self.0.borrow_mut();
        state.spoken.push(text.to_owned());
        state
            .speak_failure
            .clone()
            .map_or(Ok(()), |message| Err(SpeechFailure::new(message)))
    }
}

impl Reporter for SharedVoice {
    fn report(&mut self, message: &str) {
        self.0.borrow_mut().reports.push(message.to_owned());
    }
}
