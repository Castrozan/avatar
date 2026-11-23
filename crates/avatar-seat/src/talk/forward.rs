use super::LONGEST_SEAT_SECONDS;
use super::quoting::shell_join;
use crate::edges::EdgeFailure;
use crate::local::LocalSeat;

pub(crate) fn forward_command(local: &LocalSeat<'_>, arguments: &[String]) -> Vec<String> {
    let remote = [vec!["avatar-talk".to_owned()], arguments.to_vec()].concat();
    let reach = ["ssh", "-o", "BatchMode=yes", "-o", "ConnectTimeout=10"].map(str::to_owned);
    let host = local.config.avatar_host.clone();
    reach
        .into_iter()
        .chain([host, shell_join(&remote)])
        .collect()
}

pub(crate) fn forward(local: &LocalSeat<'_>, arguments: &[String]) -> Result<i32, EdgeFailure> {
    let command = forward_command(local, arguments);
    Ok(local
        .processes
        .run(&command, LONGEST_SEAT_SECONDS, false)?
        .code)
}
