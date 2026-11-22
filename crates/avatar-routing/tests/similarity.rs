use avatar_routing::text::similarity::ratio;
use proptest::prelude::*;

const CASES: [(&str, &str, u32); 14] = [
    ("kitten", "sitting", 4),
    ("abc", "abc", 3),
    ("abc", "xyz", 0),
    ("", "", 0),
    ("", "abc", 0),
    ("abc", "", 0),
    ("abab", "baba", 3),
    ("aabb", "bbaa", 2),
    ("xaxbx", "xbxax", 3),
    ("ástolfo", "astolfo", 6),
    ("astolfo", "ástolfo", 6),
    ("ritha", "rita", 4),
    ("shera", "shira", 4),
    ("abcdefgh", "abxdefzh", 6),
];

fn expected_ratio(matches: u32, left: &str, right: &str) -> f64 {
    let total = u32::try_from(left.chars().count() + right.chars().count()).unwrap_or(u32::MAX);
    if total == 0 {
        return 1.0;
    }
    2.0 * f64::from(matches) / f64::from(total)
}

fn assert_case(case: &(&str, &str, u32)) {
    let (left, right, matches) = *case;
    let expected = expected_ratio(matches, left, right);
    assert_eq!(
        ratio(left, right).to_bits(),
        expected.to_bits(),
        "{left:?} vs {right:?}"
    );
}

fn patterned(multiplier: usize, length: usize) -> String {
    let letters = ['a', 'b', 'c', 'd', 'e'];
    (0..length)
        .map(|index| letters[(index * multiplier + index / 3) % 5])
        .collect()
}

#[test]
fn ratio_matches_python_difflib() {
    CASES.iter().for_each(assert_case);
}

#[test]
fn ratio_applies_autojunk_when_right_is_long() {
    let (left, right) = (patterned(2, 150), patterned(3, 230));
    assert_eq!(
        ratio(&left, &right).to_bits(),
        expected_ratio(1, &left, &right).to_bits()
    );
}

proptest! {
    #[test]
    fn ratio_is_bounded_and_reflexive(text in ".{0,400}", other in ".{0,400}") {
        let score = ratio(&text, &other);
        prop_assert!((0.0..=1.0).contains(&score));
        prop_assert_eq!(ratio(&text, &text).to_bits(), 1.0_f64.to_bits());
    }
}
