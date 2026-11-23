use super::ScribeClient;
use super::answer::{language_code_of, probability_text, text_of};
use crate::error::TranscriptionError;
use crate::http::HttpSender;
use crate::segment::clean_transcript;
use crate::spoken_languages::{
    SpokenLanguage, drop_unspoken_segment, is_written_in_one_of, languages_named_by,
    spoken_language,
};
use serde_json::Value;

impl<S: HttpSender> ScribeClient<S> {
    pub async fn transcribe(
        &mut self,
        pcm: &[u8],
        language: Option<&str>,
        diagnostics: &mut Vec<String>,
    ) -> Result<String, TranscriptionError> {
        let spoken = match language.filter(|code| !code.is_empty()) {
            Some(code) => pinned_text(self, pcm, code, diagnostics).await?,
            None => detected_text(self, pcm, diagnostics).await?,
        };
        Ok(clean_transcript(&spoken))
    }
}

async fn pinned_text<S: HttpSender>(
    client: &ScribeClient<S>,
    pcm: &[u8],
    code: &str,
    diagnostics: &mut Vec<String>,
) -> Result<String, TranscriptionError> {
    let Some(language) = spoken_language(code) else {
        return Ok(drop_unspoken_segment(
            &format!("the session language {code}"),
            diagnostics,
        ));
    };
    let answer = client.answer(pcm, Some(language.pinning_code())).await?;
    heard_text(&answer, &[language], diagnostics)
}

async fn detected_text<S: HttpSender>(
    client: &mut ScribeClient<S>,
    pcm: &[u8],
    diagnostics: &mut Vec<String>,
) -> Result<String, TranscriptionError> {
    let answer = client.answer(pcm, None).await?;
    match language_code_of(&answer) {
        Some(code) if languages_named_by(Some(code)).is_empty() => {
            repinned_text(client, pcm, &answer, diagnostics).await
        }
        detected => accepted_detection(client, &answer, detected, diagnostics),
    }
}

fn accepted_detection<S: HttpSender>(
    client: &mut ScribeClient<S>,
    answer: &Value,
    detected: Option<&str>,
    diagnostics: &mut Vec<String>,
) -> Result<String, TranscriptionError> {
    let languages = languages_named_by(detected);
    let text = heard_text(answer, &languages, diagnostics)?;
    if detected.is_some() && !text.is_empty() {
        remember_first(client, &languages);
    }
    Ok(text)
}

fn remember_first<S: HttpSender>(client: &mut ScribeClient<S>, languages: &[SpokenLanguage]) {
    if let Some(&first) = languages.first() {
        client.recent_language.remember(first);
    }
}

async fn repinned_text<S: HttpSender>(
    client: &ScribeClient<S>,
    pcm: &[u8],
    answer: &Value,
    diagnostics: &mut Vec<String>,
) -> Result<String, TranscriptionError> {
    let detected = language_code_of(answer).unwrap_or_default();
    let pinning = client.recent_language.pinning_code();
    diagnostics.push(format!(
        "Scribe detected {detected} ({}), pinning {pinning}",
        probability_text(answer)
    ));
    pinned_text(client, pcm, pinning, diagnostics).await
}

fn heard_text(
    answer: &Value,
    languages: &[SpokenLanguage],
    diagnostics: &mut Vec<String>,
) -> Result<String, TranscriptionError> {
    let text = text_of(answer)?;
    if is_written_in_one_of(text, languages) {
        return Ok(text.to_owned());
    }
    let heard_as = format!(
        "{} ({})",
        language_code_of(answer).unwrap_or("None"),
        probability_text(answer)
    );
    Ok(drop_unspoken_segment(&heard_as, diagnostics))
}
