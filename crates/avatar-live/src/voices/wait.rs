use avatar_floor::addressee::Addressee;

use crate::boundaries::{Clock, HolderLookup};
use crate::session::DEFAULT_VOICE;

pub const HOLDER_WAIT_SECONDS: f64 = 15.0;
pub const HOLDER_POLL_SECONDS: f64 = 0.1;

async fn wait_for_holder(
    holders: &mut impl HolderLookup,
    pace: &mut impl Clock,
    deadline: f64,
) -> Option<Addressee> {
    let mut holder = holders.current();
    while holder.is_none() && pace.now() < deadline {
        pace.sleep(HOLDER_POLL_SECONDS).await;
        holder = holders.current();
    }
    holder
}

pub async fn voice_of_the_room_holder(
    holders: &mut impl HolderLookup,
    pace: &mut impl Clock,
    voice_for: impl Fn(&str) -> String,
) -> String {
    let deadline = pace.now() + HOLDER_WAIT_SECONDS;
    let holder = wait_for_holder(holders, pace, deadline).await;
    holder.map_or_else(
        || DEFAULT_VOICE.to_owned(),
        |found| voice_for(&found.servant),
    )
}
