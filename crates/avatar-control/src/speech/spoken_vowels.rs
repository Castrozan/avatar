use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VowelShape {
    Aa,
    Ee,
    Ih,
    Oh,
    Ou,
}

impl VowelShape {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Aa => "aa",
            Self::Ee => "ee",
            Self::Ih => "ih",
            Self::Oh => "oh",
            Self::Ou => "ou",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AlignedCharacter<'text> {
    pub text: &'text str,
    pub start: f64,
    pub end: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpokenVowel {
    pub shape: VowelShape,
    pub start: f64,
    pub end: f64,
}

#[must_use]
pub fn spoken_vowels(characters: &[AlignedCharacter<'_>]) -> Vec<SpokenVowel> {
    shapes_of(characters)
        .zip(characters)
        .filter_map(|(shape, character)| vowel_of(shape, character))
        .collect()
}

fn shapes_of<'text>(
    characters: &'text [AlignedCharacter<'_>],
) -> impl Iterator<Item = Option<VowelShape>> + 'text {
    characters.iter().scan(false, |inside_tag, character| {
        Some(shape_in_context(inside_tag, character.text))
    })
}

fn vowel_of(shape: Option<VowelShape>, character: &AlignedCharacter<'_>) -> Option<SpokenVowel> {
    shape.map(|shape| SpokenVowel {
        shape,
        start: character.start,
        end: character.end,
    })
}

fn shape_in_context(inside_tag: &mut bool, text: &str) -> Option<VowelShape> {
    *inside_tag = *inside_tag || text == "[";
    let shape = (!*inside_tag).then(|| vowel_shape(text)).flatten();
    *inside_tag = *inside_tag && text != "]";
    shape
}

fn vowel_shape(text: &str) -> Option<VowelShape> {
    let base = text.nfd().next()?;
    match base.to_lowercase().to_string().as_str() {
        "a" => Some(VowelShape::Aa),
        "e" => Some(VowelShape::Ee),
        "i" | "y" => Some(VowelShape::Ih),
        "o" => Some(VowelShape::Oh),
        "u" => Some(VowelShape::Ou),
        _ => None,
    }
}
