use crate::engine::{schedule::Schedule, state::State};
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[cfg(feature = "parallel")]
fn schedule_step_u64(schedule: &Schedule) -> u64 {
    schedule.step as u64
}

#[cfg(not(feature = "parallel"))]
fn schedule_step_u64(schedule: &Schedule) -> u64 {
    schedule.step
}

/// Timing information collected while running an initialized state.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RunStats {
    pub run_duration: f32,
    pub executed_steps: u64,
}

/// Run one initialized state using the provided schedule until `end_condition`.
///
/// This helper does not call `State::init`; callers retain full control over state setup.
pub fn run_initialized_state(state: &mut dyn State, schedule: &mut Schedule) -> RunStats {
    let timer = Instant::now();
    let start_step = schedule_step_u64(schedule);

    while !state.end_condition(schedule) {
        schedule.step(state.as_state_mut());
    }

    RunStats {
        run_duration: timer.elapsed().as_secs_f32(),
        executed_steps: schedule_step_u64(schedule).saturating_sub(start_step),
    }
}

/// Run one initialized state using the provided schedule until `end_condition`
/// or `max_steps`, whichever comes first.
///
/// This helper does not call `State::init`; callers retain full control over state setup.
pub fn run_initialized_state_bounded(
    state: &mut dyn State,
    schedule: &mut Schedule,
    max_steps: u64,
) -> RunStats {
    let timer = Instant::now();
    let start_step = schedule_step_u64(schedule);
    let stop_at = start_step.saturating_add(max_steps);

    while schedule_step_u64(schedule) < stop_at && !state.end_condition(schedule) {
        schedule.step(state.as_state_mut());
    }

    RunStats {
        run_duration: timer.elapsed().as_secs_f32(),
        executed_steps: schedule_step_u64(schedule).saturating_sub(start_step),
    }
}
