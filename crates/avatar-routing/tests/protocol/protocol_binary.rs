use serde_json::{Value, json};
use std::error::Error;
use std::io::Write;
use std::process::{Child, Command, Stdio};

fn spawn_binary() -> Result<Child, Box<dyn Error>> {
    Ok(Command::new(env!("CARGO_BIN_EXE_avatar-routing"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?)
}

fn feed(child: &mut Child, input: &str) -> Result<(), Box<dyn Error>> {
    let mut stdin = child.stdin.take().ok_or("stdin not piped")?;
    Ok(stdin.write_all(input.as_bytes())?)
}

fn parse_lines(stdout: &[u8]) -> Result<Vec<Value>, Box<dyn Error>> {
    let lines = std::str::from_utf8(stdout)?
        .lines()
        .map(serde_json::from_str)
        .collect::<Result<Vec<Value>, _>>()?;
    Ok(lines)
}

fn run_binary(input: &str) -> Result<Vec<Value>, Box<dyn Error>> {
    let mut child = spawn_binary()?;
    feed(&mut child, input)?;
    let output = child.wait_with_output()?;
    parse_lines(&output.stdout)
}

const INPUT: &str = r#"{"op":"folded","text":"Ástolfo"}
{"op":"ratio","left":"abc","right":"abc"}
not json
"#;

#[test]
fn binary_answers_each_line_and_keeps_going_after_an_error() -> Result<(), Box<dyn Error>> {
    let lines = run_binary(INPUT)?;
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], json!({"ok": "astolfo"}));
    assert_eq!(lines[1], json!({"ok": 1.0}));
    assert!(lines[2].get("error").is_some());
    Ok(())
}
