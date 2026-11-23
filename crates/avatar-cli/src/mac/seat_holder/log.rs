use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Stdio;

use avatar_seat::holder::Reporter;
use jiff::Zoned;

const SEAT_LOG: &str = ".local/share/avatar/seat-holder.log";
const LONGEST_SEAT_LOG_BYTES: u64 = 1_000_000;
const TIMESTAMP: &str = "%Y-%m-%d %H:%M:%S";

pub(super) struct SeatLog {
    file: Option<File>,
}

pub(super) fn seat_log_path(home: &Path) -> PathBuf {
    home.join(SEAT_LOG)
}

pub(super) fn open_seat_log(path: &Path) -> io::Result<File> {
    if let Some(directory) = path.parent() {
        fs::create_dir_all(directory)?;
    }
    let too_long = fs::metadata(path).is_ok_and(|found| found.len() > LONGEST_SEAT_LOG_BYTES);
    let mut options = OpenOptions::new();
    options.create(true).write(true);
    options.append(!too_long).truncate(too_long);
    options.open(path)
}

fn unavailable_log(error: &io::Error) -> SeatLog {
    let log = SeatLog { file: None };
    log.report(&error.to_string());
    log
}

fn write_line(file: Option<&File>, line: &str) -> io::Result<()> {
    match file {
        Some(mut file) => file.write_all(line.as_bytes()),
        None => io::stderr().write_all(line.as_bytes()),
    }
}

impl SeatLog {
    pub(super) fn open(home: &Path) -> Self {
        let opened = open_seat_log(&seat_log_path(home));
        opened.map_or_else(
            |error| unavailable_log(&error),
            |file| Self { file: Some(file) },
        )
    }

    pub(super) fn tunnel_stderr(&self) -> Stdio {
        let duplicate = self.file.as_ref().and_then(|file| file.try_clone().ok());
        duplicate.map_or_else(Stdio::inherit, Stdio::from)
    }
}

impl Reporter for SeatLog {
    fn report(&self, message: &str) {
        let line = format!("{} {message}\n", Zoned::now().strftime(TIMESTAMP));
        let _ = write_line(self.file.as_ref(), &line);
    }
}
