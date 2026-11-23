pub(super) const DESK_REPLY: &str = concat!(
    "speak with avatar-talk say, and hear the reply through avatar-talk follow in a ",
    "background watcher. When the room dismisses you, such as 'you can go now', say ",
    "a short goodbye and run avatar-talk close, which hands the room back to the wake."
);

pub(super) const MEETING_SETTING: &str = concat!(
    "You now hold the floor of the meeting the avatar transcribes: answer aloud ",
    "with avatar-command speak, which the meeting hears, and read it through ",
    "avatar-listen follow in a background watcher."
);

pub(super) const REMOTE_ROOM: &str = concat!(
    "the room is on machine {room}, so start every avatar command with ",
    "AVATAR_ROOM={room}, which sends it there as you: "
);

pub(super) const REMOTE_DESK_REPLY: &str = concat!(
    "speak with {reach} avatar-talk say \"...\", and hear the reply through {reach} ",
    "avatar-talk follow in a background watcher. When the room dismisses you, such ",
    "as 'you can go now', say a short goodbye and run {reach} avatar-talk close, ",
    "which hands the room back to the wake."
);

pub(super) const REMOTE_MEETING: &str = concat!(
    "answer aloud with {reach} avatar-talk say \"...\", which the meeting hears, ",
    "and read it through {reach} avatar-talk follow in a background watcher."
);

pub(super) const OPENING_LEAD: &str = "An avatar conversation is opening: ";
pub(super) const LISTENING_LEAD: &str = "The open avatar conversation now listens for you: ";
pub(super) const REMOTE_MEETING_LEAD: &str =
    "You now hold the floor of the meeting the avatar transcribes, and ";
