use super::VOICE_TAG;
use super::utterance::next_utterance;
use super::watch::{Addressing, ListeningWatch};
use crate::talk::cli::arguments::{AskArguments, WaitArguments};
use crate::talk::failure::TalkFailure;
use crate::talk::ports::{Boundaries, ConsumerLock, Files};
use crate::talk::speech::{CompletedSpeech, mark_spoken, speech_argv, spoken_in_turn};
use crate::talk::state::addressee::current;
use crate::talk::state::this_pane;
use crate::turn_taking::WITHHELD_STATUS;

const FOLLOW_SLICE_SECONDS: f64 = 30.0;

fn with_consumer<Boundary: ConsumerLock>(
    boundary: &mut Boundary,
    body: impl FnOnce(&mut Boundary) -> Result<(), TalkFailure>,
) -> Result<(), TalkFailure> {
    boundary.acquire()?;
    let result = body(boundary);
    boundary.release();
    result
}

fn print_next_utterance(
    boundary: &mut impl Boundaries,
    wait_seconds: f64,
    watch: &mut ListeningWatch,
) -> Result<(), TalkFailure> {
    if let Some(utterance) = next_utterance(boundary, wait_seconds, watch)? {
        boundary.print_out(&format!("{VOICE_TAG} {utterance}\n"));
    }
    Ok(())
}

pub fn hear(boundary: &mut impl Boundaries, hearing: &WaitArguments) -> Result<(), TalkFailure> {
    with_consumer(boundary, |locked| {
        let mut watch = ListeningWatch::new(Addressing::Always);
        print_next_utterance(locked, hearing.wait, &mut watch)
    })
}

pub fn listen_after(
    boundary: &mut impl Boundaries,
    completed: &CompletedSpeech,
    wait_seconds: f64,
) -> Result<(), TalkFailure> {
    completed.check()?;
    if completed.status_text().as_deref() == Some(WITHHELD_STATUS) {
        boundary.print_out(&completed.stdout);
    }
    let mut watch = ListeningWatch::new(Addressing::Always);
    print_next_utterance(boundary, wait_seconds, &mut watch)
}

pub fn ask(boundary: &mut impl Boundaries, asking: &AskArguments) -> Result<(), TalkFailure> {
    with_consumer(boundary, |locked| {
        mark_spoken(locked);
        let argv = speech_argv(locked, &asking.speech)?;
        let completed = spoken_in_turn(locked, argv)?;
        listen_after(locked, &completed, asking.wait.wait)
    })
}

fn follow_while_addressed(
    boundary: &mut impl Boundaries,
    watch: &mut ListeningWatch,
) -> Result<(), TalkFailure> {
    while watch.still_addressed(boundary)? {
        print_next_utterance(boundary, FOLLOW_SLICE_SECONDS, watch)?;
    }
    Ok(())
}

fn successor_name(files: &mut impl Files) -> Result<String, TalkFailure> {
    let successor = current(files)?;
    Ok(successor.map_or_else(|| "another session".to_owned(), |next| next.servant))
}

pub fn follow(boundary: &mut impl Boundaries) -> Result<(), TalkFailure> {
    let mut watch = ListeningWatch::new(Addressing::Pane(this_pane(boundary)));
    with_consumer(boundary, |locked| {
        follow_while_addressed(locked, &mut watch)
    })?;
    let name = successor_name(boundary)?;
    let line = format!("{VOICE_TAG} The room turned to {name}; this follow has ended.\n");
    boundary.print_out(&line);
    Ok(())
}
