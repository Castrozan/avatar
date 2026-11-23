use std::error::Error;
use std::process::Command;

#[test]
fn command_subcommand_prints_the_usage_for_an_unknown_command_without_connecting()
-> Result<(), Box<dyn Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_avatar"))
        .args(["command", "bogus"])
        .output()?;
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).starts_with("Usage: avatar-command status"));
    assert!(output.stdout.is_empty());
    Ok(())
}

#[test]
fn command_subcommand_accepts_hyphenated_words_as_arguments() -> Result<(), Box<dyn Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_avatar"))
        .args(["command", "--help-me"])
        .output()?;
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).starts_with("Usage: avatar-command"));
    Ok(())
}
