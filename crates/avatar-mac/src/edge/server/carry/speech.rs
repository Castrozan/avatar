use std::ops::Range;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::extract::ws::{Message, WebSocket};
use tokio::time::sleep;

use super::frames::{finish, next_frame};
use crate::edge::server::state::SeatState;
use crate::frame::{Frame, whole_samples};
use crate::seat::{PacingStep, SpeechPacing, SpeechTicket};

fn send_chunk(state: &SeatState, samples: &[u8], span: Range<usize>) {
    state.send_to_page(Message::Binary(samples[span].to_vec().into()));
}

fn next_pause(
    state: &SeatState,
    ticket: &SpeechTicket,
    pacing: &mut SpeechPacing,
    samples: &[u8],
    origin: Instant,
) -> Option<f64> {
    let interrupted = ticket.is_hushed() || !state.page_present();
    match pacing.next(interrupted) {
        PacingStep::Finished => None,
        PacingStep::Cushion(seconds) => Some(seconds),
        PacingStep::Send { start, end } => {
            send_chunk(state, samples, start..end);
            Some(pacing.pause_after_send(origin.elapsed().as_secs_f64()))
        }
    }
}

async fn play(state: &SeatState, ticket: &SpeechTicket, pcm: &[u8]) {
    let samples = whole_samples(pcm);
    let origin = Instant::now();
    let mut pacing = SpeechPacing::new(samples.len(), 0.0);
    while let Some(seconds) = next_pause(state, ticket, &mut pacing, samples, origin) {
        sleep(Duration::from_secs_f64(seconds.max(0.0))).await;
    }
}

pub(in crate::edge::server) async fn carry_speech(state: Arc<SeatState>, mut socket: WebSocket) {
    let ticket = state.begin_speech();
    if let Some(Frame::Binary(pcm)) = next_frame(&mut socket).await {
        play(&state, &ticket, &pcm).await;
    }
    finish(socket).await;
}
