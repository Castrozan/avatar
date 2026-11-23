use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use avatar_adapters::commands::FlockConsumerLock;
use avatar_conversation::talk::ports::ConsumerLock;

use crate::failure::message_of;
use crate::tools::TestResult;

const BUSY: &str = "Another avatar-talk hear, ask or follow is already listening";

fn mode(path: &Path) -> Result<u32, std::io::Error> {
    Ok(fs::metadata(path)?.permissions().mode() & 0o777)
}

#[test]
fn a_second_consumer_is_refused_until_the_first_releases() -> TestResult {
    let directory = tempfile::tempdir()?;
    let mut first = FlockConsumerLock::new(directory.path());
    let mut second = FlockConsumerLock::new(directory.path());
    assert!(first.acquire().is_ok());
    assert_eq!(message_of(second.acquire().err()), BUSY);
    first.release();
    assert!(second.acquire().is_ok());
    Ok(())
}

#[test]
fn the_lock_file_is_private_and_created_on_demand() -> TestResult {
    let directory = tempfile::tempdir()?;
    let state = directory.path().join("runtime");
    assert!(FlockConsumerLock::new(&state).acquire().is_ok());
    assert_eq!(mode(&state.join("heard.lock"))?, 0o600);
    assert_eq!(mode(&state)?, 0o700);
    Ok(())
}
