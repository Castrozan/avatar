use avatar_floor::addressee::Addressee;
use avatar_live::boundaries::{AgentTransport, CommandOutput};
use futures_util::future::OptionFuture;
use serde_json::{Value, json};
use tokio::sync::oneshot;
use tokio::task::yield_now;

pub(super) struct Replies {
    send: CommandOutput,
    status: CommandOutput,
    notify: CommandOutput,
}

pub(super) struct ScriptedTransport {
    replies: Replies,
    stop: Option<oneshot::Sender<()>>,
    pub(super) commands: Vec<Vec<String>>,
}

fn output(code: Option<i32>, text: &str) -> CommandOutput {
    CommandOutput {
        code,
        output: text.to_owned(),
    }
}

impl Replies {
    fn reporting(status: &Value) -> Self {
        Self {
            send: output(Some(0), "task-7\n"),
            status: output(Some(0), &status.to_string()),
            notify: output(Some(0), "{}"),
        }
    }

    pub(super) fn completed(task_output: &str) -> Self {
        Self::reporting(&json!({"state": "completed", "output": task_output}))
    }

    pub(super) fn working(task_output: &str) -> Self {
        Self::reporting(&json!({"state": "working", "output": task_output}))
    }

    pub(super) fn failed(message: &str) -> Self {
        Self::reporting(&json!({"state": "failed", "output": "", "errorMessage": message}))
    }

    pub(super) fn refused() -> Self {
        Self {
            send: output(Some(1), "a2a: peer refused the task\n"),
            ..Self::working("")
        }
    }

    pub(super) fn silent() -> Self {
        Self {
            send: output(None, "a2a gave no reply within 20 s"),
            ..Self::working("")
        }
    }

    pub(super) fn unable_to_notify(self, failure: &str) -> Self {
        Self {
            notify: output(Some(1), failure),
            ..self
        }
    }
}

impl ScriptedTransport {
    pub(super) fn new(replies: Replies) -> Self {
        Self {
            replies,
            stop: None,
            commands: Vec::new(),
        }
    }

    pub(super) fn stopping_at_the_first_status(replies: Replies) -> (Self, oneshot::Receiver<()>) {
        let (sender, receiver) = oneshot::channel();
        let transport = Self {
            stop: Some(sender),
            ..Self::new(replies)
        };
        (transport, receiver)
    }

    fn raise_the_stop(&mut self) -> bool {
        self.stop.take().is_some_and(|stop| stop.send(()).is_ok())
    }

    async fn status(&mut self) -> CommandOutput {
        let raised = self.raise_the_stop();
        OptionFuture::from(raised.then(yield_now)).await;
        self.replies.status.clone()
    }
}

impl AgentTransport for ScriptedTransport {
    async fn run(&mut self, _holder: &Addressee, arguments: &[String]) -> CommandOutput {
        self.commands.push(arguments.to_vec());
        match arguments.first().map(String::as_str) {
            Some("send") => self.replies.send.clone(),
            Some("notify") => self.replies.notify.clone(),
            _ => self.status().await,
        }
    }
}
