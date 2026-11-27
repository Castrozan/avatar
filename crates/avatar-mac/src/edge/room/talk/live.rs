use avatar_conversation::talk::TalkFailure;
use avatar_conversation::talk::cli::arguments::OpenArguments;
use avatar_conversation::talk::effects::live::{LiveStage, open_live_conversation_on};
use avatar_conversation::talk::ports::Boundaries;
use avatar_conversation::talk::state::seat::seat_name;
use avatar_floor::quiet::Seat;

use crate::edge::failure::talk_failure;
use crate::failure::Failure;
use crate::talk::{LiveOpening, LiveStageHooks, TalkRequest};

pub struct MacLiveOpening<Boundary> {
    pub boundary: Boundary,
}

struct HookStage<'hooks>(&'hooks mut dyn LiveStageHooks);

fn hook_failure(failure: &Failure) -> TalkFailure {
    TalkFailure::new(failure.message.clone())
}

impl<Boundary> LiveStage<Boundary> for HookStage<'_> {
    fn bring_up(&mut self, _boundary: &mut Boundary, seat: Seat) -> Result<(), TalkFailure> {
        self.0
            .bring_up(seat_name(seat))
            .map_err(|failure| hook_failure(&failure))
    }

    fn wait_until_seated(&mut self, _boundary: &mut Boundary) -> Result<(), TalkFailure> {
        self.0
            .wait_until_seated()
            .map_err(|failure| hook_failure(&failure))
    }

    fn close(&mut self, _boundary: &mut Boundary) -> Result<(), TalkFailure> {
        self.0.close().map_err(|failure| hook_failure(&failure))
    }
}

fn open_arguments(request: &TalkRequest) -> OpenArguments {
    OpenArguments {
        character: request.character.clone(),
        agent: request.agent.clone(),
        brief: request.brief.clone(),
        live: true,
        ..OpenArguments::default()
    }
}

impl<Boundary: Boundaries> LiveOpening for MacLiveOpening<Boundary> {
    fn open_live_conversation(
        &mut self,
        request: &TalkRequest,
        hooks: &mut dyn LiveStageHooks,
    ) -> Result<(), Failure> {
        let open = open_arguments(request);
        open_live_conversation_on(&mut self.boundary, &open, &mut HookStage(hooks))
            .map_err(|error| talk_failure(&error))
    }
}
