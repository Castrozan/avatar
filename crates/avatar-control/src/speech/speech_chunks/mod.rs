mod packing;
mod splitters;

use unicode_segmentation::UnicodeSegmentation;

use super::spoken_language::{AnalysisError, SpokenLanguage, detect};
use packing::packed;
use splitters::split_text;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChunkLanguage {
    Heard(SpokenLanguage),
    Undecided,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Piece {
    text: String,
    language: Option<ChunkLanguage>,
}

impl Piece {
    fn new(text: String, language: Option<ChunkLanguage>) -> Self {
        Self { text, language }
    }
}

#[derive(Debug, Clone, Copy)]
enum Splitter {
    Clause,
    WordStart,
    Grapheme,
    CodePoint,
}

const SPLITTERS: [Splitter; 4] = [
    Splitter::Clause,
    Splitter::WordStart,
    Splitter::Grapheme,
    Splitter::CodePoint,
];

pub fn split_into_chunks(text: &str, maximum: usize) -> Result<Vec<String>, AnalysisError> {
    let pieces = text
        .split_sentence_bounds()
        .map(|sentence| sentence_pieces(sentence, maximum))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(packed(pieces.into_iter().flatten().collect(), maximum)
        .into_iter()
        .map(|piece| piece.text)
        .collect())
}

fn sentence_pieces(sentence: &str, maximum: usize) -> Result<Vec<Piece>, AnalysisError> {
    let language = language_of_sentence(sentence)?;
    Ok(parts_of(
        Piece::new(sentence.to_owned(), language),
        maximum,
        0,
    ))
}

fn language_of_sentence(sentence: &str) -> Result<Option<ChunkLanguage>, AnalysisError> {
    let as_portuguese = detect(sentence, SpokenLanguage::PortugueseBrazil)?;
    let as_english = detect(sentence, SpokenLanguage::EnglishUs)?;
    Ok(match (as_portuguese == as_english, as_portuguese) {
        (true, language) => language.map(ChunkLanguage::Heard),
        (false, _) => Some(ChunkLanguage::Undecided),
    })
}

fn parts_of(piece: Piece, maximum: usize, level: usize) -> Vec<Piece> {
    let fits = utf16_length(&piece.text) <= maximum;
    match (fits, SPLITTERS.get(level)) {
        (true, _) | (false, None) => vec![piece],
        (false, Some(splitter)) => packed(split_piece(&piece, *splitter, maximum, level), maximum),
    }
}

fn split_piece(piece: &Piece, splitter: Splitter, maximum: usize, level: usize) -> Vec<Piece> {
    split_text(&piece.text, splitter)
        .into_iter()
        .flat_map(|text| parts_of(Piece::new(text, piece.language), maximum, level + 1))
        .collect()
}

fn utf16_length(text: &str) -> usize {
    text.encode_utf16().count()
}
