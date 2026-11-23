mod seat;
mod socket;

use std::fs;

use serde_json::Value;
use tokio::runtime::Builder;

use crate::machine::TestResult;
use seat::{
    end_on_request, health, host_known, seat_machine, serves_index, start,
    start_with_embedded_page, wait_until,
};
use socket::carry_through_the_audio_socket;

#[test]
fn the_seat_serves_its_embedded_page_when_no_directory_is_given() -> TestResult {
    let machine = seat_machine()?;
    let seat = start_with_embedded_page(&machine)?;
    assert!(wait_until(|| serves_index(seat.port)));
    assert!(machine.path("data/seat-page/seat.js").is_file());
    Ok(())
}

#[test]
fn the_seat_reports_what_the_host_is_doing_and_ends_on_request() -> TestResult {
    let machine = seat_machine()?;
    let mut seat = start(&machine)?;
    assert!(wait_until(|| host_known(&seat)));
    assert_eq!(health(seat.port).ok_or("no health")?["attached"], false);
    assert!(end_on_request(&mut seat)?);
    Ok(())
}

#[test]
fn a_machine_that_takes_the_seat_has_its_voice_played_and_hears_the_levels_and_the_avatar()
-> TestResult {
    let machine = seat_machine()?;
    let seat = start(&machine)?;
    assert!(wait_until(|| host_known(&seat)));
    let runtime = Builder::new_current_thread().enable_all().build()?;
    let heard = runtime.block_on(carry_through_the_audio_socket(seat.port))?;
    assert_eq!(serde_json::from_str::<Value>(&heard.levels)?["agent"], 0.5);
    assert!(wait_until(
        || fs::read(machine.path("heard")).is_ok_and(|bytes| bytes == [7, 8, 9])
    ));
    Ok(())
}
