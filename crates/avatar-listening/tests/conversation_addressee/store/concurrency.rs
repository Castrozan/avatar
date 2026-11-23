use std::io;
use std::path::Path;
use std::thread::{self, JoinHandle};

use avatar_listening::conversation_addressee::file_store::FileAddresseeStore;
use avatar_listening::conversation_addressee::store::AddresseeStore;
use tempfile::TempDir;

use super::addressee;
use crate::support::Outcome;

const STAGED_FILE: &str = ".addressee.json";
const WRITER_COUNT: usize = 4;
const WRITER_ROUNDS: i64 = 200;

type Writer = JoinHandle<io::Result<()>>;

fn write_round(store: &FileAddresseeStore, index: usize, round: i64) -> io::Result<()> {
    if round % 3 == 0 {
        store.forget()
    } else {
        store.hand_to(&addressee(&format!("%{index}"), "s", round, None))
    }
}

fn writer_rounds(store: &FileAddresseeStore, index: usize) -> io::Result<()> {
    (0..WRITER_ROUNDS).try_for_each(|round| write_round(store, index, round))
}

fn spawn_writer(state: &Path, index: usize) -> Writer {
    let store = FileAddresseeStore::new(state);
    thread::spawn(move || writer_rounds(&store, index))
}

fn read_while_writing(reader: &FileAddresseeStore, writers: &[Writer]) -> Outcome {
    while !writers.iter().all(JoinHandle::is_finished) {
        reader.current()?;
    }
    Ok(())
}

fn join_writers(writers: Vec<Writer>) -> Outcome {
    for writer in writers {
        writer.join().map_err(|_| "writer panicked")??;
    }
    Ok(())
}

fn check_settled(state: &Path, reader: &FileAddresseeStore) -> Outcome {
    assert!(!state.join(STAGED_FILE).exists());
    reader.forget()?;
    assert_eq!(reader.current()?, None);
    Ok(())
}

#[test]
fn concurrent_hand_to_and_forget_never_expose_a_partial_file() -> Outcome {
    let root = TempDir::new()?;
    let state = root.path().join("state");
    let reader = FileAddresseeStore::new(&state);
    let writers: Vec<Writer> = (0..WRITER_COUNT)
        .map(|index| spawn_writer(&state, index))
        .collect();
    read_while_writing(&reader, &writers)?;
    join_writers(writers)?;
    check_settled(&state, &reader)
}
