//! Pure timing logic: which file to loop, and when to start it.
//!
//! clipboard-tts writes the mp3, rewrites it to add an id3 tag, then plays it
//! once. So one new file produces several write events, and playback starts
//! right after the last one. We wait for the events to go quiet (debounce),
//! then wait the clip's duration before starting the loop.

use std::path::PathBuf;
use std::time::{Duration, Instant};

#[derive(Debug, PartialEq)]
pub enum Action {
    /// Stop the running loop now — a new clip is about to play once.
    StopLoop,
    /// Events went quiet; measure this file's duration and call `schedule`.
    Measure(PathBuf),
    /// One-time playback is over; loop this file.
    StartLoop(PathBuf),
}

enum State {
    Idle,
    /// Write events still arriving. `last` is the time of the latest one.
    Settling {
        path: PathBuf,
        last: Instant,
    },
    Measuring {
        path: PathBuf,
        last: Instant,
    },
    Waiting {
        path: PathBuf,
        start_at: Instant,
    },
}

pub struct Scheduler {
    state: State,
    debounce: Duration,
    margin: Duration,
}

impl Scheduler {
    pub fn new(debounce: Duration, margin: Duration) -> Self {
        Self {
            state: State::Idle,
            debounce,
            margin,
        }
    }

    /// A new or rewritten audio file. The newest file always wins.
    pub fn on_event(&mut self, path: PathBuf, now: Instant) -> Option<Action> {
        let already_stopped = matches!(&self.state, State::Settling { .. });
        self.state = State::Settling { path, last: now };
        (!already_stopped).then_some(Action::StopLoop)
    }

    /// Called with the measured duration after `Action::Measure`.
    pub fn schedule(&mut self, duration: Duration) {
        if let State::Measuring { path, last } = &mut self.state {
            let start_at = *last + duration + self.margin;
            self.state = State::Waiting {
                path: std::mem::take(path),
                start_at,
            };
        }
    }

    pub fn next_deadline(&self) -> Option<Instant> {
        match &self.state {
            State::Settling { last, .. } => Some(*last + self.debounce),
            State::Waiting { start_at, .. } => Some(*start_at),
            State::Idle | State::Measuring { .. } => None,
        }
    }

    pub fn poll(&mut self, now: Instant) -> Option<Action> {
        if self.next_deadline().is_none_or(|d| now < d) {
            return None;
        }
        match std::mem::replace(&mut self.state, State::Idle) {
            State::Settling { path, last } => {
                self.state = State::Measuring {
                    path: path.clone(),
                    last,
                };
                Some(Action::Measure(path))
            }
            State::Waiting { path, .. } => Some(Action::StartLoop(path)),
            other => {
                self.state = other;
                None
            }
        }
    }
}

// ─── unit tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
#[path = "loop_schedule_tests.rs"]
mod tests;
