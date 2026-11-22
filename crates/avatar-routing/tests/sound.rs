use avatar_routing::text::sound::sound_key;

const CASES: [(&str, &str); 27] = [
    ("Rita", "Rita"),
    ("Scheherazade", "Serasade"),
    ("chá", "Sá"),
    ("Xuanzang", "Xuansang"),
    ("Quetzalcoatl", "Quetsalkoatl"),
    ("phone", "fone"),
    ("cidade", "sidade"),
    ("zebra", "sebra"),
    ("Hokusai", "Hokusai"),
    ("queijo", "kueijo"),
    ("Ozzy", "Osi"),
    ("Kingprotea", "Kingprotea"),
    ("r", ""),
    ("rr", "r"),
    ("hhh", ""),
    ("yellow", "ielow"),
    ("", ""),
    ("ch", "S"),
    ("Shira", "Sira"),
    ("ushiwakamaru", "uSiwakamaru"),
    ("aaabbb", "ab"),
    ("a\n\nb", "a\n\nb"),
    ("ástolfo", "ástolfo"),
    ("Ritinha", "Ritina"),
    ("schsh", "S"),
    ("qq", "k"),
    ("xxcx", "SkS"),
];

#[test]
fn sound_key_matches_python_reference() {
    CASES.iter().for_each(assert_case);
}

fn assert_case(case: &(&str, &str)) {
    let (word, expected) = *case;
    assert_eq!(sound_key(word), expected, "word: {word:?}");
}
