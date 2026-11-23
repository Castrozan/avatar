mod activity;
pub mod exec;
mod files;
mod floor;
mod listener;
pub mod process;
mod system;
mod units;

pub use activity::{ActivityLog, ConsumerLock};
pub use files::Files;
pub use floor::{AgentPane, Floor};
pub use listener::{ListenerSession, ListeningRequest};
pub use process::{Invocation, ProcessFailure, ProcessOutput, Subprocess};
pub use system::{Clock, Console, Environment};
pub use units::{Readiness, Units};

pub trait Boundaries:
    Clock
    + Subprocess
    + Console
    + Environment
    + Files
    + Units
    + ListenerSession
    + Floor
    + ActivityLog
    + ConsumerLock
{
}

impl<Candidate> Boundaries for Candidate where
    Candidate: Clock
        + Subprocess
        + Console
        + Environment
        + Files
        + Units
        + ListenerSession
        + Floor
        + ActivityLog
        + ConsumerLock
{
}
