use std::future::Future;
use std::sync::Arc;

use avatar_control::synthesis::contract::{
    GeneratedSpeech, SpeechGenerator, SpeechRequest, SynthesisError, SynthesisResult,
};
use serde_json::{Value, json};

use crate::runtime::script::Script;
use crate::runtime::shared::{Shared, Waiting, new_gate, ready_waiting};

pub(crate) struct FakeGenerator {
    pub(crate) shared: Shared,
    pub(crate) script: Arc<Script>,
}

fn generation_entry(request: &SpeechRequest) -> Value {
    json!({"generate": {
        "text": request.text, "outputId": request.output_id,
        "voice": request.voice, "emotion": request.emotion, "model": request.model,
    }})
}

fn outcome(fake: &FakeGenerator, request: &SpeechRequest) -> SynthesisResult {
    match fake.script.failure_for(&request.output_id) {
        Some(message) => Err(SynthesisError::Failed(message.clone())),
        None => Ok(speech(fake, &request.output_id)),
    }
}

fn speech(fake: &FakeGenerator, output_id: &str) -> GeneratedSpeech {
    GeneratedSpeech {
        audio_path: format!("/fake/{output_id}.wav").into(),
        duration_seconds: 0.0,
        vowels: fake.script.vowels(),
        source: fake.script.source_for(output_id),
    }
}

fn generation_wait(fake: &FakeGenerator, request: &SpeechRequest) -> Waiting {
    fake.shared.record(generation_entry(request));
    if !fake.script.holds("generate") {
        return ready_waiting();
    }
    let (gate, waiting) = new_gate();
    let id = request.output_id.clone();
    fake.shared
        .with(|recording| recording.gates.generation.insert(id, gate));
    waiting
}

async fn finish_after(waiting: Waiting, result: SynthesisResult) -> SynthesisResult {
    waiting.await;
    result
}

impl SpeechGenerator for FakeGenerator {
    fn synthesize(&self, request: SpeechRequest) -> impl Future<Output = SynthesisResult> + Send {
        finish_after(generation_wait(self, &request), outcome(self, &request))
    }
}
