use crate::fakes::{assert_stored, claim_on, directory_holding, session};
use avatar_floor::addressee::Addressee;
use std::error::Error;
use std::io;
use std::path::PathBuf;
use std::thread::JoinHandle;

type ClaimOutcome = io::Result<Option<Addressee>>;

fn claim_in_thread(directory: PathBuf, pane: &'static str) -> JoinHandle<ClaimOutcome> {
    std::thread::spawn(move || {
        let sessions = vec![session("p1", "s1", None), session("p2", "s2", None)];
        claim_on(&directory, pane, sessions, true)
    })
}

fn join(handle: JoinHandle<ClaimOutcome>) -> ClaimOutcome {
    handle
        .join()
        .map_err(|_| io::Error::other("claim thread panicked"))?
}

fn race_once() -> Result<(), Box<dyn Error>> {
    let directory = directory_holding(None)?;
    let first = claim_in_thread(directory.path().to_path_buf(), "p1");
    let second = claim_in_thread(directory.path().to_path_buf(), "p2");
    let first_result = join(first)?;
    let second_result = join(second)?;
    assert_eq!(first_result, second_result);
    assert_stored(directory.path(), first_result.as_ref())
}

#[test]
fn two_concurrent_claims_leave_exactly_one_holder() -> Result<(), Box<dyn Error>> {
    for _ in 0..20 {
        race_once()?;
    }
    Ok(())
}
