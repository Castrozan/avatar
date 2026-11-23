use avatar_floor::quiet::ARM_CHECK_SECONDS;

use super::boundaries::{Boundaries, Microphone};
use super::bridge::armed;
use super::capture::Capture;
use super::machine::{Phase, WakeListener};
use super::{following, hearing};

pub(super) fn advance(
    listener: &mut WakeListener,
    boundaries: Boundaries,
    microphone: &mut dyn Microphone,
) {
    let current = std::mem::replace(&mut listener.phase, Phase::Choosing);
    listener.phase = match current {
        Phase::Choosing => choose(listener, boundaries, microphone),
        Phase::Following { check_at } => following::follow(listener, boundaries, check_at),
        Phase::Capturing(capture) => capturing(listener, boundaries, microphone, capture),
    };
}

fn choose(
    listener: &mut WakeListener,
    boundaries: Boundaries,
    microphone: &mut dyn Microphone,
) -> Phase {
    if boundaries.seat.conversation_listening() {
        return following::begin(listener, boundaries);
    }
    if !armed(&listener.cache, boundaries) {
        boundaries.effects.sleep(ARM_CHECK_SECONDS);
        return Phase::Choosing;
    }
    microphone.start();
    Phase::Capturing(Capture::default())
}

fn capturing(
    listener: &mut WakeListener,
    boundaries: Boundaries,
    microphone: &mut dyn Microphone,
    capture: Capture,
) -> Phase {
    let Some(frame) = microphone.next_frame() else {
        microphone.stop();
        return Phase::Choosing;
    };
    feed(listener, boundaries, microphone, (capture, frame))
}

fn feed(
    listener: &mut WakeListener,
    boundaries: Boundaries,
    microphone: &mut dyn Microphone,
    (mut capture, frame): (Capture, Vec<u8>),
) -> Phase {
    if let Some(segment) = capture.heard(frame) {
        hearing::hear(listener, boundaries, &segment);
        return still_armed(listener, boundaries, microphone, Capture::default());
    }
    if capture.arm_check_due() {
        return still_armed(listener, boundaries, microphone, capture);
    }
    Phase::Capturing(capture)
}

fn still_armed(
    listener: &WakeListener,
    boundaries: Boundaries,
    microphone: &mut dyn Microphone,
    capture: Capture,
) -> Phase {
    if armed(&listener.cache, boundaries) {
        return Phase::Capturing(capture);
    }
    microphone.stop();
    Phase::Choosing
}
