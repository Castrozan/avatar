use super::run_avatar;
use std::error::Error;

const INPUT: &str = r#"{"op":"quiet.run_script","script":[{"start":[{"sequence":1,"text":"a"}]},{"new_entries":[{"sequence":1,"text":"a"},{"sequence":2,"text":"b"}]}]}
{"op":"nope.unknown"}

not json
"#;

#[test]
fn floor_subcommand_matches_the_floor_library_loop() -> Result<(), Box<dyn Error>> {
    let mut expected = Vec::new();
    avatar_floor::protocol::json_lines::run_json_lines(INPUT.as_bytes(), &mut expected)?;
    assert_eq!(run_avatar(&["floor"], INPUT)?, expected);
    Ok(())
}
