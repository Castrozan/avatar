use tokio::io::{AsyncBufReadExt, BufReader, Lines};
use tokio::process::ChildStderr;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

pub(super) fn forward_diagnostics(stderr: ChildStderr) -> UnboundedReceiver<String> {
    let (diagnostics, receiver) = unbounded_channel();
    tokio::spawn(relay_lines(BufReader::new(stderr).lines(), diagnostics));
    receiver
}

async fn relay_lines(
    mut lines: Lines<BufReader<ChildStderr>>,
    diagnostics: UnboundedSender<String>,
) {
    while let Ok(Some(line)) = lines.next_line().await {
        send_diagnostic(&diagnostics, &line);
    }
}

fn send_diagnostic(diagnostics: &UnboundedSender<String>, line: &str) {
    let _ = diagnostic_line(line).map(|diagnostic| diagnostics.send(diagnostic));
}

fn diagnostic_line(line: &str) -> Option<String> {
    let trimmed = line.trim();
    (!trimmed.is_empty() && !trimmed.starts_with("frame=")).then(|| trimmed.to_owned())
}
