use avatar_seat::meetings::camera::{CameraStore, FileSystemStore};

type Outcome<Value> = Result<Value, Box<dyn std::error::Error>>;

fn written_report(root: &std::path::Path) -> Outcome<String> {
    let directory = format!("{}/a/b", root.display());
    FileSystemStore.make_directory(&directory)?;
    let report = format!("{directory}/report");
    FileSystemStore.write_text(&report, "active")?;
    Ok(report)
}

#[test]
fn the_store_writes_reads_and_removes_under_created_directories() -> Outcome<()> {
    let root = tempfile::tempdir()?;
    let report = written_report(root.path())?;
    assert_eq!(
        FileSystemStore.read_text(&report).as_deref(),
        Some("active")
    );
    FileSystemStore.remove(&report)?;
    assert!(!FileSystemStore.exists(&report));
    Ok(FileSystemStore.remove(&report)?)
}

#[test]
fn reading_a_missing_report_finds_nothing() {
    assert_eq!(FileSystemStore.read_text("/nonexistent/report"), None);
}
