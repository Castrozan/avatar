mod process;
mod wav;

use std::path::{Path, PathBuf};

use process::{feed_text, piper_outcome, spawn_piper};

use crate::synthesis::contract::{
    GeneratedSpeech, SpeechGenerator, SpeechRequest, SpeechSource, SynthesisError, voice_or,
};

pub async fn prepare_output_root_directory(path: &Path) -> Result<(), SynthesisError> {
    tokio::fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(path)
        .await?;
    Ok(())
}

#[derive(Debug, Clone)]
pub struct TextToSpeechGenerator {
    program: PathBuf,
    output_root_directory: PathBuf,
    default_voice: String,
    voice_model_directory: PathBuf,
}

impl TextToSpeechGenerator {
    #[must_use]
    pub fn new(
        program: PathBuf,
        output_root_directory: PathBuf,
        default_voice: String,
        voice_model_directory: PathBuf,
    ) -> Self {
        Self {
            program,
            output_root_directory,
            default_voice,
            voice_model_directory,
        }
    }

    async fn prepare_audio_path(&self, output_id: &str) -> Result<PathBuf, SynthesisError> {
        let output_directory = self.output_root_directory.join(output_id);
        tokio::fs::create_dir_all(&output_directory).await?;
        Ok(output_directory.join("voice.wav"))
    }

    async fn run_piper(
        &self,
        model_path: &Path,
        audio_path: &Path,
        text: &str,
    ) -> Result<(), SynthesisError> {
        let mut child = spawn_piper(&self.program, model_path, audio_path)?;
        feed_text(&mut child, text).await?;
        piper_outcome(child.wait_with_output().await?.status)
    }
}

impl SpeechGenerator for TextToSpeechGenerator {
    async fn synthesize(&self, request: SpeechRequest) -> Result<GeneratedSpeech, SynthesisError> {
        let voice = validated_voice(voice_or(request.voice.as_deref(), &self.default_voice))?;
        let audio_path = self.prepare_audio_path(&request.output_id).await?;
        let model_path = self.voice_model_directory.join(format!("{voice}.onnx"));
        self.run_piper(&model_path, &audio_path, &request.text)
            .await?;
        let duration_seconds = read_duration(&audio_path).await?;
        Ok(piper_speech(audio_path, duration_seconds))
    }
}

async fn read_duration(audio_path: &Path) -> Result<f64, SynthesisError> {
    let audio = tokio::fs::read(audio_path).await?;
    wav::duration_seconds(&audio)
}

fn piper_speech(audio_path: PathBuf, duration_seconds: f64) -> GeneratedSpeech {
    GeneratedSpeech {
        audio_path,
        duration_seconds,
        vowels: Vec::new(),
        source: SpeechSource::Piper,
    }
}

fn validated_voice(voice: &str) -> Result<&str, SynthesisError> {
    if is_voice_name(voice) {
        return Ok(voice);
    }
    Err(SynthesisError::UnknownVoice(voice.to_owned()))
}

fn is_voice_name(voice: &str) -> bool {
    !voice.is_empty()
        && voice.chars().all(|character| {
            character.is_ascii_alphanumeric() || character == '_' || character == '-'
        })
}
