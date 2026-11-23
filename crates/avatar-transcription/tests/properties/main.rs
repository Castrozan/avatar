use avatar_routing::names::vocabulary::Vocabulary;
use avatar_transcription::hex::transcription_path;
use avatar_transcription::scribe::{
    KEYTERMS_BEFORE_MINIMUM_BILLING, RECOGNIZED_TERMS, keyterms, multipart_body,
};
use avatar_transcription::segment::{clean_transcript, wav_bytes};
use proptest::prelude::*;

fn fixed_terms() -> Vec<String> {
    RECOGNIZED_TERMS
        .iter()
        .map(|term| (*term).to_owned())
        .collect()
}

proptest! {
    #[test]
    fn cleaning_is_idempotent_and_single_spaced(text in ".{0,80}") {
        let cleaned = clean_transcript(&text);
        prop_assert_eq!(clean_transcript(&cleaned), cleaned.clone());
        prop_assert!(!cleaned.contains("  "));
        prop_assert_eq!(cleaned.trim(), cleaned.as_str());
    }

    #[test]
    fn wav_header_describes_the_pcm_it_wraps(pcm in proptest::collection::vec(any::<u8>(), 0..256)) {
        let wav = wav_bytes(&pcm);
        prop_assert_eq!(wav.len(), 44 + pcm.len());
        let length_field = u32::try_from(pcm.len()).unwrap_or(u32::MAX).to_le_bytes();
        prop_assert_eq!(&wav[40..44], length_field.as_slice());
        prop_assert_eq!(&wav[44..], pcm.as_slice());
    }

    #[test]
    fn keyterms_keep_fixed_terms_first_and_cap_the_list(
        names in proptest::collection::vec("[A-Za-z][A-Za-z ]{0,10}", 0..150),
    ) {
        let roster = names.iter().fold(String::new(), |roster, name| format!("{roster}{name} | role\n"));
        let terms = keyterms(Some(&roster), None::<&Vocabulary>);
        prop_assert!(terms.len() <= KEYTERMS_BEFORE_MINIMUM_BILLING);
        let fixed = fixed_terms();
        prop_assert_eq!(&terms[..RECOGNIZED_TERMS.len()], fixed.as_slice());
        let unique: std::collections::HashSet<&String> = terms.iter().collect();
        prop_assert_eq!(unique.len(), terms.len());
    }

    #[test]
    fn multipart_body_closes_with_the_boundary(
        boundary in "[a-z0-9]{8,20}",
        value in "[ -~]{0,20}",
        wav in proptest::collection::vec(any::<u8>(), 0..64),
    ) {
        let fields = vec![("keyterms".to_owned(), value.clone())];
        let body = multipart_body(&boundary, &fields, &wav);
        let closing = format!("\r\n--{boundary}--\r\n");
        prop_assert!(body.ends_with(closing.as_bytes()));
        prop_assert!(value.is_empty() || body.windows(value.len()).any(|window| window == value.as_bytes()));
    }

    #[test]
    fn transcription_path_has_no_raw_spaces_and_language_only_when_named(
        model in "[a-z_0-9 ]{1,12}",
        language in proptest::option::of("[a-zA-Z -]{0,6}"),
    ) {
        let path = transcription_path(&model, language.as_deref());
        prop_assert!(!path.contains(' '));
        let named = language.as_deref().is_some_and(|code| !code.is_empty());
        prop_assert_eq!(path.contains("&language="), named);
    }
}
