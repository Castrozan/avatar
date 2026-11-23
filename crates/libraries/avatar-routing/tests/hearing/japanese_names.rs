use super::japanese_names_table::CASES;
use avatar_routing::hearing::name_hearing::heard_name;
use avatar_routing::names::vocabulary::Vocabulary;
use std::error::Error;

const ORDINARY_WORDS: &str = "the and tell hear wake check build name moon night see mesa ana ontem fala tudo bem ok hey funny anyone yesterday";

fn ordinary_vocabulary() -> Vocabulary {
    ORDINARY_WORDS
        .split_whitespace()
        .enumerate()
        .map(|(rank, word)| (word.to_owned(), rank))
        .collect()
}

fn windows_of(field: &str) -> Vec<Vec<String>> {
    field
        .split(';')
        .filter(|window| !window.trim().is_empty())
        .map(|window| window.split_whitespace().map(str::to_owned).collect())
        .collect()
}

fn names_of(field: &str) -> Vec<String> {
    field
        .split(',')
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .collect()
}

fn expected_of(field: &str) -> Option<(String, usize)> {
    let (name, size) = field.split_once(':')?;
    Some((name.to_owned(), size.parse().ok()?))
}

fn check_row(vocabulary: &Vocabulary, row: &str) -> Result<(), Box<dyn Error>> {
    let fields: Vec<&str> = row.split('|').map(str::trim).collect();
    let with_vocabulary = (fields[2] == "y").then_some(vocabulary);
    let found = heard_name(
        &windows_of(fields[0]),
        &names_of(fields[1]),
        with_vocabulary,
    );
    ensure_agreement(found == expected_of(fields[3]), row)
}

fn ensure_agreement(agrees: bool, row: &str) -> Result<(), Box<dyn Error>> {
    if agrees {
        Ok(())
    } else {
        Err(format!("disagrees with Python: {row}").into())
    }
}

#[test]
fn japanese_names_agree_with_python_reference() -> Result<(), Box<dyn Error>> {
    let vocabulary = ordinary_vocabulary();
    for row in CASES.lines().filter(|line| !line.trim().is_empty()) {
        check_row(&vocabulary, row)?;
    }
    Ok(())
}
