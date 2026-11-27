use std::future::Future;

use super::super::POLL_SECONDS;
use super::super::watch::AnswerWatch;
use super::dispatch::Dispatch;
use crate::boundaries::{AgentTransport, Courier};

pub(super) trait Poller {
    fn is_open(&self) -> bool;
    fn poll_once(&mut self) -> impl Future<Output = bool>;
}

pub(super) async fn poll_until_answered(poller: &mut impl Poller) -> bool {
    let mut answered = false;
    while !answered && poller.is_open() {
        answered = poller.poll_once().await;
    }
    answered
}

pub(super) struct Polling<'run, 'a, Transport: AgentTransport, Crew: Courier> {
    pub(super) dispatch: &'run mut Dispatch<'a, Transport, Crew>,
    pub(super) watch: AnswerWatch,
    pub(super) task_id: String,
    pub(super) tag: String,
}

impl<'run, 'a, Transport: AgentTransport, Crew: Courier> Polling<'run, 'a, Transport, Crew> {
    pub(super) fn start(
        dispatch: &'run mut Dispatch<'a, Transport, Crew>,
        task_id: String,
        tag: String,
    ) -> Self {
        let watch = AnswerWatch::new(dispatch.reporter.crew.now());
        Self {
            dispatch,
            watch,
            task_id,
            tag,
        }
    }
}

impl<Transport: AgentTransport, Crew: Courier> Poller for Polling<'_, '_, Transport, Crew> {
    fn is_open(&self) -> bool {
        self.watch.is_open(self.dispatch.reporter.crew.now())
    }

    async fn poll_once(&mut self) -> bool {
        self.dispatch.reporter.crew.sleep(POLL_SECONDS).await;
        let verdict = self.dispatch.status_verdict(&self.task_id, &self.tag).await;
        let observation = self
            .watch
            .observe(self.dispatch.reporter.crew.now(), verdict);
        self.dispatch.announce(observation).await
    }
}
