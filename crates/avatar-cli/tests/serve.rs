use std::error::Error;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::{Child, ChildStdout, Command, Stdio};

type TestResult = Result<(), Box<dyn Error>>;

struct Site {
    child: Child,
    address: String,
    _stdout: BufReader<ChildStdout>,
}

impl Site {
    fn command() -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_avatar"));
        command.args(["serve", "--address", "127.0.0.1:0"]);
        command
    }

    fn start(root: &std::path::Path) -> Result<Self, Box<dyn Error>> {
        let mut command = Self::command();
        command.env("AVATAR_RENDERER_ROOT", root);
        Self::spawn(command)
    }

    fn start_embedded(data: &std::path::Path) -> Result<Self, Box<dyn Error>> {
        let mut command = Self::command();
        command.env_remove("AVATAR_RENDERER_ROOT");
        command.env("AVATAR_DATA_DIRECTORY", data);
        Self::spawn(command)
    }

    fn spawn(mut command: Command) -> Result<Self, Box<dyn Error>> {
        let mut child = command.stdout(Stdio::piped()).spawn()?;
        let stdout = BufReader::new(child.stdout.take().ok_or("stdout")?);
        Self::announced(child, stdout)
    }

    fn announced(child: Child, mut stdout: BufReader<ChildStdout>) -> Result<Self, Box<dyn Error>> {
        let mut line = String::new();
        stdout.read_line(&mut line)?;
        Ok(Self {
            child,
            address: line.trim().to_owned(),
            _stdout: stdout,
        })
    }
}

impl Drop for Site {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn get(address: &str, headers: &str) -> Result<String, Box<dyn Error>> {
    let mut stream = TcpStream::connect(address)?;
    let request = format!(
        "GET /renderer.js HTTP/1.1\r\nHost: localhost\r\n{headers}Connection: close\r\n\r\n"
    );
    stream.write_all(request.as_bytes())?;
    let mut raw = String::new();
    stream.read_to_string(&mut raw)?;
    Ok(raw)
}

#[test]
fn serve_subcommand_tells_the_browser_never_to_keep_a_copy() -> TestResult {
    let root = tempfile::tempdir()?;
    std::fs::write(root.path().join("renderer.js"), "current")?;
    let site = Site::start(root.path())?;
    let answer = get(&site.address, "")?;
    assert!(answer.contains("cache-control: no-store"));
    assert!(answer.ends_with("current"));
    Ok(())
}

#[test]
fn serve_subcommand_replaces_a_stale_copy_despite_identical_timestamps() -> TestResult {
    let root = tempfile::tempdir()?;
    std::fs::write(root.path().join("renderer.js"), "current")?;
    let site = Site::start(root.path())?;
    let answer = get(
        &site.address,
        "If-Modified-Since: Thu, 01 Jan 2099 00:00:00 GMT\r\n",
    )?;
    assert!(answer.starts_with("HTTP/1.1 200"));
    assert!(answer.ends_with("current"));
    Ok(())
}

#[test]
fn serve_subcommand_serves_the_embedded_renderer_without_any_variable() -> TestResult {
    let data = tempfile::tempdir()?;
    let site = Site::start_embedded(data.path())?;
    let answer = get(&site.address, "")?;
    assert!(answer.starts_with("HTTP/1.1 200"));
    assert!(answer.contains("from \"three\""));
    assert!(data.path().join("renderer/index.html").is_file());
    assert!(data.path().join("renderer/portraits/jarvis.jpeg").is_file());
    Ok(())
}
