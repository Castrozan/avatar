use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use avatar_seat::meetings::drivers::{DriverStore, FileSystemDrivers};

type Outcome<Value> = Result<Value, Box<dyn std::error::Error>>;

fn packaged_driver(root: &Path, name: &str, body: &str) -> Outcome<PathBuf> {
    let driver = root.join("store").join(name);
    fs::create_dir_all(driver.join("Contents"))?;
    fs::write(driver.join("Contents/driver"), body)?;
    Ok(driver)
}

fn read(driver: &Path, file: &str) -> Outcome<String> {
    Ok(fs::read_to_string(driver.join("Contents").join(file))?)
}

fn linked_install(root: &Path) -> Outcome<PathBuf> {
    let packaged = packaged_driver(root, "A.driver", "two")?;
    symlink("driver", packaged.join("Contents/alias"))?;
    let installed = root.join("HAL/A.driver");
    FileSystemDrivers.replace(&packaged, &installed)?;
    Ok(installed)
}

#[test]
fn replacing_copies_the_packaged_driver_with_its_links() -> Outcome<()> {
    let root = tempfile::tempdir()?;
    let installed = linked_install(root.path())?;
    let alias = fs::read_link(installed.join("Contents/alias"))?;
    assert_eq!(
        (read(&installed, "driver")?, alias),
        ("two".to_owned(), "driver".into())
    );
    Ok(())
}

fn stale_install(root: &Path) -> Outcome<(PathBuf, PathBuf)> {
    let packaged = packaged_driver(root, "A.driver", "old")?;
    let installed = root.join("HAL/A.driver");
    FileSystemDrivers.replace(&packaged, &installed)?;
    fs::write(packaged.join("Contents/driver"), "new")?;
    fs::write(installed.join("Contents/stale"), "left over")?;
    Ok((packaged, installed))
}

#[test]
fn replacing_removes_what_the_new_driver_no_longer_has() -> Outcome<()> {
    let root = tempfile::tempdir()?;
    let (packaged, installed) = stale_install(root.path())?;
    FileSystemDrivers.replace(&packaged, &installed)?;
    assert_eq!(read(&installed, "driver")?, "new");
    assert!(!installed.join("Contents/stale").exists());
    Ok(())
}
