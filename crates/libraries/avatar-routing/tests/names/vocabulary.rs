use avatar_routing::names::vocabulary::{Vocabulary, ranks_from_paths};
use std::path::{Path, PathBuf};

const FIRST_FILE: &str = "\n  \nHéllo extra\nworld\n\n Zebra\nthe 1\nCafé\n";
const SECOND_FILE: &str = "WORLD\nhello\ncafe\n\n  x\n";
const MERGED_RANKS: &[(&str, usize)] = &[
    ("cafe", 2),
    ("hello", 0),
    ("the", 3),
    ("world", 0),
    ("x", 3),
    ("zebra", 2),
];
const SINGLE_RANKS: &[(&str, usize)] = &[
    ("cafe", 4),
    ("hello", 0),
    ("the", 3),
    ("world", 1),
    ("zebra", 2),
];

fn fixture_directory(test_name: &str) -> PathBuf {
    let directory = std::env::temp_dir().join(format!(
        "avatar-routing-vocabulary-{}-{test_name}",
        std::process::id()
    ));
    assert!(std::fs::create_dir_all(&directory).is_ok());
    directory
}

fn joined_fixture_paths(test_name: &str, files: &[(&str, &str)]) -> String {
    let directory = fixture_directory(test_name);
    files
        .iter()
        .map(|(name, contents)| write_fixture(&directory, name, contents))
        .collect::<Vec<_>>()
        .join(":")
}

fn write_fixture(directory: &Path, name: &str, contents: &str) -> String {
    let path = directory.join(name);
    assert!(std::fs::write(&path, contents).is_ok());
    path.to_string_lossy().into_owned()
}

fn expected(pairs: &[(&str, usize)]) -> Vocabulary {
    pairs
        .iter()
        .map(|(word, rank)| ((*word).to_owned(), *rank))
        .collect()
}

#[test]
fn ranks_merge_across_files_keeping_the_minimum() {
    let paths = joined_fixture_paths("merge", &[("a.txt", FIRST_FILE), ("b.txt", SECOND_FILE)]);
    assert_eq!(ranks_from_paths(&paths), Some(expected(MERGED_RANKS)));
}

#[test]
fn single_file_ranks_follow_non_blank_line_order() {
    let paths = joined_fixture_paths("single", &[("a.txt", FIRST_FILE)]);
    assert_eq!(ranks_from_paths(&paths), Some(expected(SINGLE_RANKS)));
}

#[test]
fn blank_lines_do_not_consume_ranks() {
    let paths = joined_fixture_paths("gaps", &[("g.txt", "  \n\nalpha\n\nbeta")]);
    assert_eq!(
        ranks_from_paths(&paths),
        Some(expected(&[("alpha", 0), ("beta", 1)]))
    );
}

#[test]
fn only_leading_words_are_ranked_and_folded() {
    let paths = joined_fixture_paths("fold", &[("c.txt", "CAFÉ x\ncafe\n")]);
    assert_eq!(ranks_from_paths(&paths), Some(expected(&[("cafe", 0)])));
}

#[test]
fn file_without_words_gives_empty_vocabulary() {
    let paths = joined_fixture_paths("blank", &[("bl.txt", "  \n\n")]);
    assert_eq!(ranks_from_paths(&paths), Some(expected(&[])));
}

#[test]
fn empty_path_list_gives_none() {
    assert_eq!(ranks_from_paths(""), None);
}

#[test]
fn unreadable_file_gives_none() {
    let good = joined_fixture_paths("missing", &[("ok.txt", "alpha\n")]);
    let paths = format!("{good}:/nonexistent/avatar-routing-missing.txt");
    assert_eq!(ranks_from_paths(&paths), None);
}

#[test]
fn directory_path_gives_none() {
    let directory = std::env::temp_dir();
    assert_eq!(ranks_from_paths(&directory.to_string_lossy()), None);
}
