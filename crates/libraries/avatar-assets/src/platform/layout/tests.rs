use super::*;

fn home() -> PathBuf {
    PathBuf::from("/home/someone")
}

#[test]
fn macos_uses_application_support() {
    let path = default_for(true, Some(home()), Some(PathBuf::from("/share")));
    assert_eq!(
        path,
        PathBuf::from("/home/someone/Library/Application Support/avatar")
    );
}

#[test]
fn linux_prefers_the_xdg_data_home() {
    let path = default_for(false, Some(home()), Some(PathBuf::from("/share")));
    assert_eq!(path, PathBuf::from("/share/avatar"));
}

#[test]
fn linux_falls_back_to_the_home_share_directory() {
    let path = default_for(false, Some(home()), None);
    assert_eq!(path, PathBuf::from("/home/someone/.local/share/avatar"));
}

#[test]
fn a_relative_xdg_data_home_is_ignored() {
    let path = default_for(false, Some(home()), Some(PathBuf::from("relative")));
    assert_eq!(path, PathBuf::from("/home/someone/.local/share/avatar"));
}
