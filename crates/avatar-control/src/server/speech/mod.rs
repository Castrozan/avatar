mod events;
mod port;

pub use port::{DiscardingSpeechPort, SpeechEvent, SpeechPort};

pub(crate) use events::{forward_events, outbound_for};
