use avatar_routing::hearing::wake_phrase::WakeMatch;
use avatar_routing::names::session::Session;

use super::super::boundaries::WakeEffects;
use super::super::failure::Boundary;
use super::super::records::{Delivery, report_delivery};
use super::setting::{Setting, spoken_prompt};

fn handed_the_prompt(
    effects: &dyn WakeEffects,
    session: &Session,
    text: &str,
) -> Boundary<&'static str> {
    let host = session.host.as_deref();
    if effects.prompt_now(&session.pane, text, host)? {
        return Ok("prompt");
    }
    effects.notify_at_idle(&session.pane, text, host)?;
    Ok("notify")
}

pub(in crate::listener) struct Dispatch<'a> {
    pub(in crate::listener) dry_run: bool,
    pub(in crate::listener) matched: &'a WakeMatch,
    pub(in crate::listener) setting: Setting,
    pub(in crate::listener) sequence: Option<i64>,
}

impl Dispatch<'_> {
    fn target(&self) -> String {
        let session = &self.matched.session;
        format!("{} at {}", session.servant, session.pane)
    }

    fn prompt(&self, effects: &dyn WakeEffects) -> String {
        let room = effects.room_name().unwrap_or_default();
        spoken_prompt(
            &self.matched.message,
            self.setting,
            &self.matched.session,
            &room,
        )
    }

    fn delivery<'a>(&'a self, delivered: &'a str, seconds: f64) -> Delivery<'a> {
        let session = &self.matched.session;
        Delivery {
            setting: self.setting,
            sequence: self.sequence,
            servant: &session.servant,
            pane: &session.pane,
            host: session.host.as_deref(),
            delivered,
            seconds,
        }
    }
}

pub(in crate::listener) fn deliver(
    effects: &dyn WakeEffects,
    dispatch: &Dispatch,
) -> Boundary<bool> {
    let target = dispatch.target();
    if dispatch.dry_run {
        effects.report(&format!(
            "would wake {target}: {}",
            dispatch.matched.message
        ));
        return Ok(false);
    }
    effects.report(&format!("wake {target}"));
    send(effects, dispatch).map(|()| true)
}

fn send(effects: &dyn WakeEffects, dispatch: &Dispatch) -> Boundary<()> {
    let text = dispatch.prompt(effects);
    let sent = effects.now();
    let delivered = handed_the_prompt(effects, &dispatch.matched.session, &text)?;
    let seconds = effects.now() - sent;
    report_delivery(effects, &dispatch.delivery(delivered, seconds));
    Ok(())
}
