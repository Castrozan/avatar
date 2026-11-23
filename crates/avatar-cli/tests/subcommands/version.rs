use std::error::Error;
use std::process::Command;

#[test]
fn version_flag_prints_the_package_version() -> Result<(), Box<dyn Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_avatar"))
        .arg("--version")
        .output()?;
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout)?, "avatar 0.1.0\n");
    Ok(())
}
