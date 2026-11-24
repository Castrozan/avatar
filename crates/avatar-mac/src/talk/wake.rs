use avatar_seat::talk::with_the_remote_seat;

#[must_use]
pub fn opening_where_the_wake_was_heard() -> Vec<String> {
    with_the_remote_seat(&["open".to_owned()])
}
