use super::run_avatar;
use std::error::Error;

const INPUT: &str = r#"{"op":"folded","text":"Ástolfo"}
{"op":"ratio","left":"abc","right":"abc"}

not json
"#;

#[test]
fn routing_subcommand_matches_the_routing_library_loop() -> Result<(), Box<dyn Error>> {
    let mut expected = Vec::new();
    avatar_routing::protocol::json_lines::run_json_lines(INPUT.as_bytes(), &mut expected)?;
    assert_eq!(run_avatar(&["routing"], INPUT)?, expected);
    Ok(())
}
