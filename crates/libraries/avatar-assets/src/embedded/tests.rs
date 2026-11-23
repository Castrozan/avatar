use super::*;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn the_renderer_ships_its_entry_page() {
    assert!(RENDERER.get_file("index.html").is_some());
    assert!(SEAT_PAGE.get_file("index.html").is_some());
    assert!(WAKE_PAGE.get_file("index.html").is_some());
}

#[test]
fn rewriting_identical_content_leaves_the_file_alone() -> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("nested/file.txt");
    write_if_changed(&path, b"one")?;
    let first = fs::metadata(&path)?.modified()?;
    write_if_changed(&path, b"one")?;
    assert_eq!(fs::metadata(&path)?.modified()?, first);
    write_if_changed(&path, b"two")?;
    assert_eq!(fs::read(&path)?, b"two");
    Ok(())
}

#[test]
fn a_directory_is_written_in_full() -> TestResult {
    let directory = tempfile::tempdir()?;
    write_directory(&WAKE_PAGE, directory.path())?;
    assert!(directory.path().join("wake-capture.js").is_file());
    Ok(())
}
