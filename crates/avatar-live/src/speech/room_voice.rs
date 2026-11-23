use std::collections::HashMap;

use crate::events::{LiveEvent, TaskRequest};

pub struct RoomVoiceState {
    speaking_voice: String,
    generation: usize,
    task_owners: HashMap<String, usize>,
}

fn task_of(event: &LiveEvent) -> Option<&TaskRequest> {
    match event {
        LiveEvent::TaskRequested(task) => Some(task),
        _ => None,
    }
}

fn owner_of(owners: &mut HashMap<String, usize>, task_id: &str, is_final: bool) -> Option<usize> {
    if is_final {
        return owners.remove(task_id);
    }
    owners.get(task_id).copied()
}

impl RoomVoiceState {
    #[must_use]
    pub fn new(speaking_voice: &str) -> Self {
        Self {
            speaking_voice: speaking_voice.to_owned(),
            generation: 0,
            task_owners: HashMap::new(),
        }
    }

    #[must_use]
    pub fn speaking_voice(&self) -> &str {
        &self.speaking_voice
    }

    #[must_use]
    pub fn generation(&self) -> usize {
        self.generation
    }

    #[must_use]
    pub fn wants_change(&self, wanted: Option<&str>) -> bool {
        wanted.is_some_and(|voice| !voice.is_empty() && voice != self.speaking_voice)
    }

    pub fn change_to(&mut self, speaking_voice: &str) {
        speaking_voice.clone_into(&mut self.speaking_voice);
        self.generation += 1;
    }

    pub fn accept(&mut self, generation: usize, event: &LiveEvent) -> bool {
        let current = generation == self.generation;
        let owned = task_of(event).filter(|_| current);
        self.task_owners
            .extend(owned.map(|task| (task.task_id.clone(), generation)));
        current
    }

    pub fn route_delivery(&mut self, task_id: &str, is_final: bool) -> Option<String> {
        let owner = owner_of(&mut self.task_owners, task_id, is_final);
        let owner = owner.unwrap_or(self.generation);
        (owner == self.generation).then(|| task_id.to_owned())
    }
}
