use serde_json::{Map, Value};

use super::directory::SessionDirectory;
use super::error::SessionError;
use super::{purpose_name, service_for};
use crate::conversation_state::{ListenerProbe, ListenerPurpose};
use crate::listener::runtime::capturing_services;
use crate::listener::units::SessionUnits;

pub struct SessionRequest {
    pub purpose: ListenerPurpose,
    pub sources: Map<String, Value>,
    pub settings: Map<String, Value>,
}

fn configuration_of(request: &SessionRequest) -> Map<String, Value> {
    let mut configuration = Map::new();
    configuration.insert("purpose".into(), purpose_name(request.purpose).into());
    configuration.insert("sources".into(), Value::Object(request.sources.clone()));
    configuration.extend(request.settings.clone());
    configuration
}

pub fn stop_listening(units: &mut impl SessionUnits) -> Result<(), SessionError> {
    for service in capturing_services(units) {
        units.stop(service)?;
    }
    Ok(())
}

pub fn begin_listening(
    directory: &SessionDirectory,
    units: &mut impl SessionUnits,
    request: &SessionRequest,
) -> Result<(), SessionError> {
    directory.create()?;
    stop_listening(units)?;
    directory.erase_records()?;
    directory.write_configuration(&configuration_of(request))?;
    Ok(units.start(service_for(request.purpose))?)
}

struct SessionProbe<'a, Units> {
    directory: &'a SessionDirectory,
    units: &'a mut Units,
}

impl<Units: SessionUnits> ListenerProbe for SessionProbe<'_, Units> {
    fn unit_active(&mut self, unit: &str) -> bool {
        self.units.is_active(unit)
    }

    fn configured_purpose(&mut self) -> Option<String> {
        self.directory.configured_purpose().ok().flatten()
    }
}

pub fn live_purpose(
    directory: &SessionDirectory,
    units: &mut impl SessionUnits,
) -> Option<ListenerPurpose> {
    crate::conversation_state::live_purpose(&mut SessionProbe { directory, units })
}
