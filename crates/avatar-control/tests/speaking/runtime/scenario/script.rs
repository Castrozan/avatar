use std::collections::{HashMap, HashSet};

use avatar_control::speaking::runtime::Platform;
use avatar_control::speech::spoken_vowels::SpokenVowel;
use avatar_control::synthesis::contract::SpeechSource;
use serde_json::Value;

use super::parsing::{array, failures, faults, platform_of, source_of, strings, vowel_of};

pub(crate) struct ChannelFaults {
    pub(crate) open: Option<String>,
    pub(crate) release: Option<String>,
}

pub(crate) struct Script {
    hold: HashSet<String>,
    sources: Vec<String>,
    vowels: Vec<Value>,
    failures: HashMap<String, String>,
    pub(crate) faults: ChannelFaults,
    pub(crate) platform: Platform,
    pub(crate) gate_status: Option<Value>,
}

impl Script {
    pub(crate) fn from_scenario(scenario: &Value) -> Self {
        Self {
            hold: strings(scenario, "hold").into_iter().collect(),
            sources: strings(scenario, "sources"),
            vowels: array(scenario, "vowels"),
            failures: failures(scenario),
            faults: faults(scenario),
            platform: platform_of(scenario),
            gate_status: scenario.get("gate").cloned(),
        }
    }

    pub(crate) fn holds(&self, kind: &str) -> bool {
        self.hold.contains(kind)
    }

    pub(crate) fn failure_for(&self, output_id: &str) -> Option<&String> {
        self.failures.get(output_id)
    }

    pub(crate) fn source_for(&self, output_id: &str) -> SpeechSource {
        let index: usize = output_id
            .rsplit('-')
            .next()
            .and_then(|tail| tail.parse().ok())
            .unwrap_or(0);
        let name = self.sources.get(index % self.sources.len().max(1));
        name.map_or(SpeechSource::Kokoro, |name| source_of(name))
    }

    pub(crate) fn vowels(&self) -> Vec<SpokenVowel> {
        self.vowels.iter().map(vowel_of).collect()
    }
}
