use avatar_listening::listener::runtime::listening;
use avatar_listening::listener::units::{SessionUnits, SystemdUnits};
use avatar_seat::host::state::{HostProbe, HostSnapshot, MEETING_TARGET, ProbeFailure};

pub(super) struct SystemdProbe;

impl HostProbe for SystemdProbe {
    fn read(&self) -> Result<HostSnapshot, ProbeFailure> {
        let mut units = SystemdUnits::new("systemctl");
        Ok(HostSnapshot {
            listening: listening(&mut units),
            meeting: units.is_active(MEETING_TARGET),
        })
    }
}
