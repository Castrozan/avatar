use avatar_floor::addressee::Addressee;
use avatar_live::agent::report::{AgentReport, AgentRequest};
use avatar_live::boundaries::{AgentAsker, Courier, HolderLookup, VoiceDelivery};
use serde_json::{Value, json};

pub(super) struct Holder(pub(super) Option<Addressee>);
pub(super) struct ScriptedAsker(pub(super) Vec<AgentReport>, pub(super) Vec<Value>);
pub(super) struct Voice(pub(super) Vec<Value>);

impl HolderLookup for Holder {
    fn current(&mut self) -> Option<Addressee> {
        self.0.clone()
    }
}

async fn report_all(courier: &mut impl Courier, reports: Vec<AgentReport>) {
    for report in reports {
        courier.report(report).await;
    }
}

impl AgentAsker for ScriptedAsker {
    async fn ask(&mut self, request: &AgentRequest, courier: &mut impl Courier) {
        let question = &request.question;
        self.1.push(json!([
            request.holder.pane,
            question.request,
            question.number
        ]));
        report_all(courier, self.0.clone()).await;
    }

    async fn notify(&mut self, request: &AgentRequest, reason: &str) -> Result<(), String> {
        self.1
            .push(json!(["notify", request.question.number, reason]));
        Ok(())
    }
}

impl VoiceDelivery for Voice {
    async fn deliver(&mut self, task_id: &str, result: &str, is_final: bool) {
        self.0.push(json!([task_id, result, is_final]));
    }
}
