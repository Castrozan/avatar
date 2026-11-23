use std::collections::VecDeque;
use std::rc::Rc;

use super::doubles::{Script, Ticks};
use avatar_listening::listener::daemon::{
    RecordError, TRANSCRIBER_ATTEMPTS, TranscribeFailure, transcribe_when_ready,
};

fn unreachable_script(failures: usize, then: Result<String, TranscribeFailure>) -> Script {
    let mut outcomes: VecDeque<_> = (0..failures)
        .map(|_| Err(TranscribeFailure::Unreachable("down".into())))
        .collect();
    outcomes.push_back(then);
    Script {
        outcomes,
        calls: Rc::default(),
    }
}

#[test]
fn the_transcriber_is_retried_until_it_answers() {
    let pauses = Rc::default();
    let mut timing = Ticks {
        count: 0,
        pauses: Rc::clone(&pauses),
    };
    let mut script = unreachable_script(TRANSCRIBER_ATTEMPTS - 1, Ok("late".into()));
    let text = transcribe_when_ready(&mut script, &mut timing, b"audio", Some("pt"));
    assert_eq!(text.ok().as_deref(), Some("late"));
    assert_eq!(pauses.borrow().len(), TRANSCRIBER_ATTEMPTS - 1);
}

#[test]
fn a_transcriber_that_never_answers_times_out_after_thirty_pauses() {
    let pauses = Rc::default();
    let mut timing = Ticks {
        count: 0,
        pauses: Rc::clone(&pauses),
    };
    let mut script = unreachable_script(TRANSCRIBER_ATTEMPTS, Ok("unused".into()));
    let outcome = transcribe_when_ready(&mut script, &mut timing, b"audio", None);
    assert!(matches!(outcome, Err(RecordError::NoTranscriberAnswered)));
    assert_eq!(pauses.borrow().len(), TRANSCRIBER_ATTEMPTS);
    assert_eq!(script.calls.borrow().len(), TRANSCRIBER_ATTEMPTS);
}
