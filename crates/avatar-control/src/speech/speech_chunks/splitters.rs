use unicode_segmentation::UnicodeSegmentation;

use super::Splitter;

const CLAUSE_PUNCTUATION: &str = ",;:–—";
const CJK_CLAUSE_PUNCTUATION: &str = "、，；：";

pub(super) fn split_text(text: &str, splitter: Splitter) -> Vec<String> {
    let characters: Vec<char> = text.chars().collect();
    match splitter {
        Splitter::Clause => cut_at(&characters, &clause_boundaries(&characters)),
        Splitter::WordStart => cut_at(&characters, &word_start_boundaries(&characters)),
        Splitter::Grapheme => text.graphemes(true).map(str::to_owned).collect(),
        Splitter::CodePoint => characters.iter().map(char::to_string).collect(),
    }
}

fn cut_at(characters: &[char], boundaries: &[usize]) -> Vec<String> {
    let starts = std::iter::once(0).chain(boundaries.iter().copied());
    let ends = boundaries
        .iter()
        .copied()
        .chain(std::iter::once(characters.len()));
    starts
        .zip(ends)
        .map(|(start, end)| characters[start..end].iter().collect())
        .collect()
}

fn clause_boundaries(characters: &[char]) -> Vec<usize> {
    (1..characters.len())
        .filter(|&index| {
            !is_js_space(characters[index])
                && (CJK_CLAUSE_PUNCTUATION.contains(characters[index - 1])
                    || punctuation_before_spaces(characters, index))
        })
        .collect()
}

fn punctuation_before_spaces(characters: &[char], index: usize) -> bool {
    let run_start = characters[..index]
        .iter()
        .rposition(|character| !is_js_space(*character))
        .map_or(0, |position| position + 1);
    run_start < index && run_start > 0 && CLAUSE_PUNCTUATION.contains(characters[run_start - 1])
}

fn word_start_boundaries(characters: &[char]) -> Vec<usize> {
    (1..characters.len())
        .filter(|&index| is_js_space(characters[index - 1]) && !is_js_space(characters[index]))
        .collect()
}

fn is_js_space(character: char) -> bool {
    is_ecmascript_white_space(character) || is_line_terminator(character)
}

fn is_ecmascript_white_space(character: char) -> bool {
    matches!(
        character,
        '\t' | '\u{b}' | '\u{c}' | ' ' | '\u{a0}' | '\u{1680}' | '\u{2000}'
            ..='\u{200a}' | '\u{202f}' | '\u{205f}' | '\u{3000}' | '\u{feff}'
    )
}

fn is_line_terminator(character: char) -> bool {
    matches!(character, '\n' | '\r' | '\u{2028}' | '\u{2029}')
}
