use std::path::{Path, PathBuf};

use avatar_control::synthesis::piper::TextToSpeechGenerator;

use crate::harness::{wav_fixture, write_script};

pub(super) fn capture_prologue(directory: &Path) -> String {
    let arguments = directory.join("piper-args.txt").display().to_string();
    let stdin = directory.join("piper-stdin.txt").display().to_string();
    format!(
        "printf '%s\\n' \"$@\" > '{arguments}'\nout=''\nwhile [ $# -gt 0 ]; do if [ \"$1\" = --output_file ]; then out=\"$2\"; fi; shift; done\ncat > '{stdin}'"
    )
}

pub(super) fn piper_program(directory: &Path, body: &str) -> std::io::Result<PathBuf> {
    let fixture = directory.join("fixture.wav");
    std::fs::write(&fixture, wav_fixture())?;
    let script = format!(
        "{}\n{}",
        capture_prologue(directory),
        body.replace("FIXTURE", &fixture.display().to_string())
    );
    let path = directory.join("piper");
    write_script(&path, &script)?;
    Ok(path)
}

pub(super) fn generator(directory: &Path, program: PathBuf) -> TextToSpeechGenerator {
    TextToSpeechGenerator::new(
        program,
        directory.join("tts"),
        "faber".to_owned(),
        directory.join("voices"),
    )
}

pub(super) fn copying_fixture(directory: &Path) -> std::io::Result<PathBuf> {
    piper_program(directory, "cp 'FIXTURE' \"$out\"")
}

pub(super) fn expected_arguments(root: &Path) -> String {
    let model = root.join("voices/faber.onnx");
    let output = root.join("tts/turn/voice.wav");
    format!(
        "--model\n{}\n--output_file\n{}\n",
        model.display(),
        output.display()
    )
}
