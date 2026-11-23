use super::quoting::shell_quote;

const REACH_HOST: [&str; 5] = ["ssh", "-o", "BatchMode=yes", "-o", "ConnectTimeout=10"];

pub(super) fn a2a_words(host: Option<&str>, arguments: &[String]) -> Vec<String> {
    let command = std::iter::once("a2a".to_owned()).chain(arguments.iter().cloned());
    let Some(host) = host else {
        return command.collect();
    };
    let joined: Vec<String> = command.map(|word| shell_quote(&word)).collect();
    let mut words: Vec<String> = REACH_HOST.map(str::to_owned).to_vec();
    words.extend([host.to_owned(), joined.join(" ")]);
    words
}
