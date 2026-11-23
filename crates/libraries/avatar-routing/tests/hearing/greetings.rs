use avatar_routing::hearing::wake_phrase::opens_with_greeting;
use std::error::Error;

const CASES: [(&str, bool); 15] = [
    ("Ne, Astolfo", true),
    ("Moshi moshi!", true),
    ("nene?", true),
    ("Neenee", true),
    ("Nee", true),
    ("нэ, Астольфо", true),
    ("NENE", true),
    ("moshimoshi", true),
    ("nen", false),
    ("mosh", false),
    ("shi moshi", false),
    ("Ok ne", true),
    ("ねえ、聞こえる？", false),
    ("Hey Astolfo", true),
    ("Ishtar", false),
];

fn ensure_agreement(agrees: bool, text: &str) -> Result<(), Box<dyn Error>> {
    if agrees {
        Ok(())
    } else {
        Err(format!("disagrees with Python: {text:?}").into())
    }
}

#[test]
fn greeting_opening_agrees_with_python_reference() -> Result<(), Box<dyn Error>> {
    for (text, expected) in CASES {
        ensure_agreement(opens_with_greeting(text) == expected, text)?;
    }
    Ok(())
}
