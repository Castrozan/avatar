use avatar_routing::text::japanese_sound::{japanese_sound_key, without_honorific};
use std::error::Error;

const KEY_CASES: [(&str, &str); 32] = [
    ("sutorufo", "strfo"),
    ("asutorufo", "astrfo"),
    ("katsushika", "kasika"),
    ("tsuzuki", "ski"),
    ("tzar", "sar"),
    ("quetzal", "kesar"),
    ("syuten", "sten"),
    ("shuten", "sten"),
    ("lusia", "usia"),
    ("vanesa", "banesa"),
    ("wataru", "atar"),
    ("ryu", "iu"),
    ("ru", "u"),
    ("tokyo", "tkio"),
    ("dotto", "dt"),
    ("tou", "tou"),
    ("nou", "nou"),
    ("anastasia", "anastasia"),
    ("ottsu", "ots"),
    ("sanu", "san"),
    ("sylvain", "srbain"),
    ("kitsune", "kisne"),
    ("dodo", "d"),
    ("ouo", "ouo"),
    ("uwu", "u"),
    ("kyuu", "kiu"),
    ("ts", "s"),
    ("tzu", "s"),
    ("queue", "keue"),
    ("sutoru", "str"),
    ("moshimoshi", "mosimosi"),
    ("ne", "ne"),
];

const HONORIFIC_CASES: [(&str, &str); 17] = [
    ("absan", "absan"),
    ("abcsan", "abc"),
    ("xyzkun", "xyz"),
    ("sutorufosama", "sutorufo"),
    ("sanchan", "san"),
    ("dono", "dono"),
    ("abcdono", "abc"),
    ("__ksan", "__k"),
    ("12kun", "12kun"),
    ("ñandosan", "ñando"),
    ("日本語san", "日本語"),
    ("katsushikachan", "katsushika"),
    ("kunkun", "kun"),
    ("sanssan", "sans"),
    ("aosan", "aosan"),
    ("abcSAN", "abcSAN"),
    ("ab", "ab"),
];

fn ensure_equal(actual: &str, expected: &str, input: &str) -> Result<(), Box<dyn Error>> {
    if actual == expected {
        Ok(())
    } else {
        Err(format!("{input:?}: {actual:?} != {expected:?}").into())
    }
}

#[test]
fn japanese_sound_key_agrees_with_python_reference() -> Result<(), Box<dyn Error>> {
    for (word, expected) in KEY_CASES {
        ensure_equal(&japanese_sound_key(word), expected, word)?;
    }
    Ok(())
}

#[test]
fn honorific_suffix_removal_agrees_with_python_reference() -> Result<(), Box<dyn Error>> {
    for (text, expected) in HONORIFIC_CASES {
        ensure_equal(&without_honorific(text), expected, text)?;
    }
    Ok(())
}
