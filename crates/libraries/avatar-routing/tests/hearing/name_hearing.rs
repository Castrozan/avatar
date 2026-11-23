use avatar_routing::hearing::name_hearing::heard_name;
use avatar_routing::names::vocabulary::Vocabulary;

const ORDINARY_WORDS: &str = "the and tell hear wake check build name moon night see mesa ana ontem fala tudo bem ok hey funny anyone yesterday";

const CASES: &str = r"
astolfo | astolfo,ishtar | y | astolfo:1
ishtar | astolfo,ishtar | y | ishtar:1
as tolfo | astolfo | y | -
as;tolfo | astolfo | y | astolfo:1
ana stasia | anastasia | y | anastasia:2
estolfo | astolfo | y | astolfo:1
astolfo;estolfo | astolfo | y | astolfo:1
media | medea | y | medea:1
anna | ana | y | ana:1
anna | ana | n | -
mesa | messa | y | -
mesa | messa | n | -
mesa;mesa x | messa | n | -
ishtor | ishtar,ishtir | y | -
ishtor | ishtar,ishtir | n | -
 | astolfo | y | -
astolfo |  | y | -
hey;astolfo | astolfo | y | astolfo:1
çelia | çelia | y | çelia:1
shera | scheherazade | y | -
sheherazade | scheherazade | y | scheherazade:1
tell me | tellme | y | -
ontem | ontem | y | ontem:1
ontem | ontam | y | -
anastasia;ana stasia | anastasia | y | anastasia:1
anastasiya | anastasia,anastasiia | y | -
ishtar a | ishtar | y | -
ishtar | ishtar | n | ishtar:1
";

fn ordinary_vocabulary() -> Vocabulary {
    ORDINARY_WORDS
        .split_whitespace()
        .enumerate()
        .map(|(rank, word)| (word.to_owned(), rank))
        .collect()
}

fn split_row(row: &str) -> Vec<&str> {
    row.split('|').map(str::trim).collect()
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

fn assert_row(vocabulary: &Vocabulary, row: &str) {
    let fields = split_row(row);
    let with_vocabulary = (fields[2] == "y").then_some(vocabulary);
    let found = heard_name(
        &windows_of(fields[0]),
        &names_of(fields[1]),
        with_vocabulary,
    );
    assert_eq!(found, expected_of(fields[3]), "{row}");
}

#[test]
fn heard_name_agrees_with_python_reference() {
    let vocabulary = ordinary_vocabulary();
    CASES
        .lines()
        .filter(|line| !line.trim().is_empty())
        .for_each(|row| assert_row(&vocabulary, row));
}
