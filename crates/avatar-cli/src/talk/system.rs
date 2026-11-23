use avatar_conversation::talk::ports::{
    ActivityLog, Clock, Console, Environment, Files, Invocation, ProcessFailure, ProcessOutput,
    Subprocess,
};
use serde_json::{Map, Value};

use super::boundaries::TalkBoundaries;
use super::host::{LoggedActivity, ProcessEnvironment, SystemClock, Terminal};

impl<Booted, Listening> Clock for TalkBoundaries<Booted, Listening> {
    fn monotonic(&self) -> f64 {
        SystemClock.monotonic()
    }

    fn wall(&self) -> f64 {
        SystemClock.wall()
    }

    fn sleep(&mut self, seconds: f64) {
        SystemClock.sleep(seconds);
    }
}

impl<Booted, Listening> Subprocess for TalkBoundaries<Booted, Listening> {
    fn run(&mut self, invocation: &Invocation) -> Result<ProcessOutput, ProcessFailure> {
        self.process.run(invocation)
    }

    fn relay(&mut self, invocation: &Invocation) -> Result<i32, ProcessFailure> {
        self.process.relay(invocation)
    }
}

impl<Booted, Listening> Console for TalkBoundaries<Booted, Listening> {
    fn print_out(&mut self, text: &str) {
        Terminal.print_out(text);
    }

    fn print_err(&mut self, text: &str) {
        Terminal.print_err(text);
    }
}

impl<Booted, Listening> Environment for TalkBoundaries<Booted, Listening> {
    fn variable(&self, name: &str) -> Option<String> {
        ProcessEnvironment.variable(name)
    }

    fn hostname(&self) -> String {
        ProcessEnvironment.hostname()
    }
}

impl<Booted, Listening> Files for TalkBoundaries<Booted, Listening> {
    fn exists(&mut self, name: &str) -> bool {
        self.files.exists(name)
    }

    fn read(&mut self, name: &str) -> Option<String> {
        self.files.read(name)
    }

    fn modified_at(&mut self, name: &str) -> Option<f64> {
        self.files.modified_at(name)
    }

    fn write(&mut self, name: &str, content: &str) {
        self.files.write(name, content);
    }

    fn touch(&mut self, name: &str) {
        self.files.touch(name);
    }
}

impl<Booted, Listening> ActivityLog for TalkBoundaries<Booted, Listening> {
    fn report_talk(&mut self, action: &str, pane: Option<&str>) {
        LoggedActivity.report_talk(action, pane);
    }

    fn report_speech(&mut self, pane: Option<&str>, details: &Map<String, Value>) {
        LoggedActivity.report_speech(pane, details);
    }
}
