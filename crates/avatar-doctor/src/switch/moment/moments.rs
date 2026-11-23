use std::collections::HashMap;

use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::boundary::Boundary;

#[derive(Clone, Debug, PartialEq)]
pub struct Moments(Vec<(Boundary, Option<f64>)>);

impl Moments {
    #[must_use]
    pub fn unrecorded() -> Self {
        Self(
            Boundary::ALL
                .iter()
                .map(|boundary| (*boundary, None))
                .collect(),
        )
    }

    #[must_use]
    pub fn get(&self, boundary: Boundary) -> Option<f64> {
        self.0
            .iter()
            .find(|(stored, _)| *stored == boundary)
            .and_then(|(_, value)| *value)
    }

    pub fn record_all(&mut self, pairs: impl IntoIterator<Item = (Boundary, Option<f64>)>) {
        pairs
            .into_iter()
            .for_each(|(boundary, value)| self.record(boundary, value));
    }

    fn record(&mut self, boundary: Boundary, value: Option<f64>) {
        self.0
            .iter_mut()
            .filter(|(stored, _)| *stored == boundary)
            .for_each(|slot| slot.1 = value);
    }
}

impl Serialize for Moments {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        self.0
            .iter()
            .try_for_each(|(boundary, value)| map.serialize_entry(boundary.key(), value))?;
        map.end()
    }
}

impl<'de> Deserialize<'de> for Moments {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let mut stated = HashMap::<String, Option<f64>>::deserialize(deserializer)?;
        Ok(Self(
            Boundary::ALL
                .iter()
                .map(|boundary| (*boundary, stated.remove(boundary.key()).flatten()))
                .collect(),
        ))
    }
}
