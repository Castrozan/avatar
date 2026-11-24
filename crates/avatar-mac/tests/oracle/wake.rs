use avatar_mac::talk::{carry_out, opening_where_the_wake_was_heard};
use serde_json::json;

use crate::talk::{code_outcome, edges};
use crate::world::World;

fn idle_mac_with_no_seat_attached() -> World {
    World::new(&json!({
        "active": [], "answers": {"8765": "never", "3000": "never"}, "fail": {},
        "request": {"command": "open", "language": null, "character": null, "text": "",
                    "emotion": "", "voice": null, "wait": 0},
        "phase": "closed", "forward": null, "host_code": 0,
        "seat": {"reachable": false, "listening": false, "meeting_held": false, "kept_here": false},
        "attached": "never", "pane": null, "holder": null, "addresses": true, "live_script": []
    }))
}

#[test]
fn a_wake_heard_at_this_mac_opens_at_the_host_with_the_remote_seat() {
    let mut world = idle_mac_with_no_seat_attached();
    let (mut host, mut live) = edges(&world);
    let opening = opening_where_the_wake_was_heard();
    let result = carry_out(&mut world, &mut host, &mut live, &opening);
    assert_eq!(code_outcome(&result), json!(["code", 0]));
    let carried = json!(["host_carry_out", ["open", "--seat", "remote", "+agent"]]);
    let log = world.log.entries();
    assert!(log.contains(&carried), "{log:?}");
}
