mod actor;
mod command;
mod connection;
mod listener;
mod next_command;
mod speech_port;

pub use listener::{ShutdownHandle, serve};
pub use speech_port::{DiscardingSpeechPort, SpeechPort};
