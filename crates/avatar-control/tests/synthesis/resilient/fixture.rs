use avatar_control::speech::spoken_language::SpokenLanguage;
use avatar_control::synthesis::contract::{SpeechGenerator, SpeechRequest};
use avatar_control::synthesis::resilient::{LocalVoices, LocalVoicesByLanguage, Resilient};

use crate::harness::{Log, Recorder, new_log};

pub(super) fn voices() -> LocalVoicesByLanguage {
    LocalVoicesByLanguage {
        portuguese_brazil: LocalVoices {
            kokoro: "pf_dora".to_owned(),
            piper: "faber".to_owned(),
        },
        english_us: LocalVoices {
            kokoro: "af_heart".to_owned(),
            piper: "lessac".to_owned(),
        },
    }
}

pub(super) fn recorder(log: &Log, label: &'static str, fails: bool) -> Recorder {
    Recorder {
        label,
        log: log.clone(),
        fails,
    }
}

pub(super) fn resilient(
    log: &Log,
    cloud_fails: bool,
    kokoro_fails: bool,
) -> Resilient<Recorder, Recorder, Recorder> {
    let cloud = recorder(log, "cloud", cloud_fails);
    let kokoro = recorder(log, "kokoro", kokoro_fails);
    Resilient::new(
        cloud,
        kokoro,
        recorder(log, "piper", false),
        SpokenLanguage::PortugueseBrazil,
        voices(),
    )
}

pub(super) async fn run(
    speech: SpeechRequest,
    cloud_fails: bool,
    kokoro_fails: bool,
) -> (Option<String>, Vec<String>) {
    let log = new_log();
    let outcome = resilient(&log, cloud_fails, kokoro_fails)
        .synthesize(speech)
        .await;
    let audio = outcome
        .ok()
        .map(|speech| speech.audio_path.display().to_string());
    (audio, log.lock().await.clone())
}
