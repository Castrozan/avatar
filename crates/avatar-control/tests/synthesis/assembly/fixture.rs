use std::path::{Path, PathBuf};

use avatar_control::speech::spoken_language::SpokenLanguage;
use avatar_control::synthesis::assembly::{
    CloudVoiceSettings, Configuration, LocalVoiceSettings, SpeechStack, assemble_speech_generator,
};
use avatar_control::synthesis::character::CharacterVoice;
use avatar_control::synthesis::resilient::{LocalVoices, LocalVoicesByLanguage};

use crate::harness::{
    UNAVAILABLE_ALIGNMENT, media_speech_program, success_body, wav_fixture, write_script,
};

pub(super) fn local_voices(kokoro: &str, piper: &str) -> LocalVoices {
    LocalVoices {
        kokoro: kokoro.to_owned(),
        piper: piper.to_owned(),
    }
}

pub(super) fn local_settings(directory: &Path) -> LocalVoiceSettings {
    let by_language = LocalVoicesByLanguage {
        portuguese_brazil: local_voices("pf_dora", "faber"),
        english_us: local_voices("af_heart", "lessac"),
    };
    LocalVoiceSettings {
        output_root_directory: directory.join("tts"),
        voice_model_directory: directory.join("voices"),
        by_language,
    }
}

pub(super) fn cloud_settings() -> CloudVoiceSettings {
    CloudVoiceSettings {
        voice: "cloudDefault".to_owned(),
        language: SpokenLanguage::PortugueseBrazil,
    }
}

pub(super) fn first_character() -> CharacterVoice {
    CharacterVoice {
        name: "first".to_owned(),
        cloud_voice: Some("cloudFirst".to_owned()),
        local_voice: None,
    }
}

pub(super) fn base_configuration(directory: &Path) -> Configuration {
    Configuration {
        media_speech_program: PathBuf::new(),
        piper_program: PathBuf::new(),
        cloud: cloud_settings(),
        local: local_settings(directory),
        characters: vec![first_character()],
    }
}

pub(super) fn configuration(
    directory: &Path,
    media_speech: PathBuf,
    piper: PathBuf,
) -> Configuration {
    Configuration {
        media_speech_program: media_speech,
        piper_program: piper,
        ..base_configuration(directory)
    }
}

pub(super) fn fake_piper(directory: &Path) -> std::io::Result<PathBuf> {
    let fixture = directory.join("fixture.wav");
    std::fs::write(&fixture, wav_fixture())?;
    let body = format!(
        "cat > /dev/null\nfor argument; do out=\"$argument\"; done\ncp '{}' \"$out\"",
        fixture.display()
    );
    let path = directory.join("piper");
    write_script(&path, &body)?;
    Ok(path)
}

pub(super) fn failing_providers(directory: &Path, providers: &str) -> std::io::Result<PathBuf> {
    let failure = format!("case \"$3\" in {providers}) echo down >&2; exit 1;; esac");
    media_speech_program(
        directory,
        &format!("{failure}\n{}", success_body(UNAVAILABLE_ALIGNMENT)),
    )
}

pub(super) fn stack_for(directory: &Path, media_speech: PathBuf) -> std::io::Result<SpeechStack> {
    let piper = fake_piper(directory)?;
    assemble_speech_generator(&configuration(directory, media_speech, piper))
        .map_err(|error| std::io::Error::other(error.to_string()))
}
