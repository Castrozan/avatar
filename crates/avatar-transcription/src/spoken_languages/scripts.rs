use std::ops::RangeInclusive;

const LATIN_RANGES: [RangeInclusive<char>; 8] = [
    'A'..='Z',
    'a'..='z',
    'ª'..='ª',
    'º'..='º',
    'À'..='Ö',
    'Ø'..='ö',
    'ø'..='ɏ',
    'Ḁ'..='ỿ',
];

const JAPANESE_RANGES: [RangeInclusive<char>; 9] = [
    '々'..='〇',
    'ぁ'..='ヿ',
    'ㇰ'..='ㇿ',
    '㐀'..='䶿',
    '一'..='鿿',
    '豈'..='﫿',
    'Ａ'..='Ｚ',
    'ａ'..='ｚ',
    'ｦ'..='ﾟ',
];

pub(super) fn is_latin_letter(character: char) -> bool {
    LATIN_RANGES.iter().any(|range| range.contains(&character))
}

pub(super) fn is_japanese_character(character: char) -> bool {
    JAPANESE_RANGES
        .iter()
        .any(|range| range.contains(&character))
}
