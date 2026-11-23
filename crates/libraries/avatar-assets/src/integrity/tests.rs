use super::*;

type TestResult = Result<(), Box<dyn std::error::Error>>;

const EMPTY_SHA256: &str = "sha256-47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=";

fn digest_of(integrity: &Integrity, content: &[u8]) -> io::Result<Vec<u8>> {
    let mut verifier = integrity.verifier(Vec::new());
    verifier.write_all(content)?;
    Ok(verifier.finish())
}

#[test]
fn a_verifier_digests_what_it_forwards() -> TestResult {
    let integrity = Integrity::parse(EMPTY_SHA256)?;
    assert!(integrity.matches(&digest_of(&integrity, b"")?));
    Ok(())
}

#[test]
fn other_content_does_not_match() -> TestResult {
    let integrity = Integrity::parse(EMPTY_SHA256)?;
    assert!(!integrity.matches(&digest_of(&integrity, b"x")?));
    Ok(())
}

#[test]
fn unknown_algorithms_and_bad_encodings_are_refused() {
    assert!(Integrity::parse("md5-AAAA").is_err());
    assert!(Integrity::parse("sha256-***").is_err());
    assert!(Integrity::parse("plain").is_err());
}
