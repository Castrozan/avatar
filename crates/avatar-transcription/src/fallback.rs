use crate::error::TranscriptionError;
use crate::hex::HexClient;
use crate::http::HttpSender;
use crate::scribe::ScribeClient;
use crate::spoken_languages::{drop_unspoken_segment, is_written_in_one_of, languages_named_by};

pub async fn transcribe_with_local_fallback<S: HttpSender>(
    cloud: &mut ScribeClient<S>,
    local: &HexClient<S>,
    pcm: &[u8],
    language: Option<&str>,
    diagnostics: &mut Vec<String>,
) -> Result<String, TranscriptionError> {
    match cloud.transcribe(pcm, language, diagnostics).await {
        Ok(text) => Ok(text),
        Err(error) => {
            diagnostics.push(format!("Cloud transcription failed, using Hex: {error}"));
            local_text(local, pcm, language, diagnostics).await
        }
    }
}

async fn local_text<S: HttpSender>(
    local: &HexClient<S>,
    pcm: &[u8],
    language: Option<&str>,
    diagnostics: &mut Vec<String>,
) -> Result<String, TranscriptionError> {
    let text = local.transcribe(pcm, language).await?;
    let languages = languages_named_by(language);
    if is_written_in_one_of(&text, &languages) {
        return Ok(text);
    }
    let heard_as = format!("Hex in {}", language.unwrap_or("None"));
    Ok(drop_unspoken_segment(&heard_as, diagnostics))
}
