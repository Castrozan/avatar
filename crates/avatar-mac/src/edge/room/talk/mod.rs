mod commands;
mod host;
mod live;
mod turns;

use super::MacRoom;

pub use host::MacHostTalk;
pub use live::MacLiveOpening;

impl<Talk> MacRoom<Talk> {
    pub fn with_talk<Next>(self, talk: Next) -> MacRoom<Next> {
        MacRoom {
            stage: self.stage,
            session: self.session,
            seat: self.seat,
            camera: self.camera,
            call_defaults: self.call_defaults,
            config: self.config,
            talk,
        }
    }
}
