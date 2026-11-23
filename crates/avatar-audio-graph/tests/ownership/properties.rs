use avatar_audio_graph::ownership::{AudioPhase, observe};
use avatar_audio_graph::pulse::module_record::ModuleRecord;
use avatar_audio_graph::state::record::AudioState;
use proptest::prelude::*;

fn core_module(index: u32) -> ModuleRecord {
    ModuleRecord::new(index, "module-null-sink", "sink_name=AvatarMix".to_owned())
}

proptest! {
    #[test]
    fn observed_phase_follows_the_recorded_graph_facts(
        recorded_count in 0_u32..3,
        ready in any::<bool>(),
        present_count in 0_u32..3,
        attached in any::<bool>(),
    ) {
        let recorded: Vec<ModuleRecord> = (1..=recorded_count).map(core_module).collect();
        let microphone = attached.then(|| avatar_audio_graph::state::record::MicrophoneRecord {
            source: "synthetic-human.monitor".to_owned(),
            module: ModuleRecord::new(90, "module-loopback", "source=synthetic-human.monitor".to_owned()),
        });
        let state = AudioState { modules: recorded.clone(), ready, microphone };
        let present: Vec<ModuleRecord> = recorded.iter().take(present_count as usize).cloned().collect();
        let phase = observe(&state, &present);
        if recorded.is_empty() {
            prop_assert_eq!(phase, AudioPhase::Absent);
        } else if !ready || present.len() != recorded.len() {
            prop_assert_eq!(phase, AudioPhase::Orphaned);
        } else {
            prop_assert!(matches!(phase, AudioPhase::RunningDetached | AudioPhase::RunningAttached));
        }
    }
}
