mod floor;
mod routing;
mod version;

use std::error::Error;
use std::io::Write;
use std::process::{Command, Stdio};

fn run_avatar(arguments: &[&str], input: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_avatar"))
        .args(arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    let mut stdin = child.stdin.take().ok_or("stdin not piped")?;
    stdin.write_all(input.as_bytes())?;
    drop(stdin);
    Ok(child.wait_with_output()?.stdout)
}
