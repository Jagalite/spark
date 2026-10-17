//! Caller-fed adapters for counted progress, explicit task state, and stream activity.
//!
//! These helpers never fetch data, poll a source, read a clock, or transition a task.
//! Store them in host state and pass their values to ordinary catalogue samples.
//! Stream activity is a bounded visual signal, not throughput, remaining work, or
//! evidence that an operation is finished.
//!
//! ```
//! use spark_catalog::adapters::{CountedProgress, TaskSnapshot, TaskStatus};
//!
//! let counts = CountedProgress::new(7, Some(10))?;
//! let task = TaskSnapshot::new(TaskStatus::Running, counts.progress());
//! assert_eq!(task.status(), TaskStatus::Running);
//! assert!(task.progress().value().unwrap() <= 0.7);
//! # Ok::<(), spark_catalog::adapters::AdapterError>(())
//! ```

use crate::{Level, Progress};
use spark_runtime::{HostTime, Period};
use std::fmt;

/// An immutable count snapshot. Counts and task status remain independent.
///
/// Unknown totals and the valid empty count `0 / 0` produce unknown progress.
/// A known nonzero total produces the greatest representable `f64` no larger
/// than the exact ratio, so rounding never reports completion early. Keep the
/// integer counts for authoritative labels; art is a conservative approximation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CountedProgress {
    completed: u64,
    total: Option<u64>,
}

impl CountedProgress {
    /// Rejects counts greater than a known total, without silently clamping.
    pub const fn new(completed: u64, total: Option<u64>) -> Result<Self, AdapterError> {
        if let Some(total) = total
            && completed > total
        {
            return Err(AdapterError::CountExceedsTotal);
        }
        Ok(Self { completed, total })
    }

    pub const fn completed(self) -> u64 {
        self.completed
    }

    pub const fn total(self) -> Option<u64> {
        self.total
    }

    pub fn progress(self) -> Progress {
        match self.total {
            None | Some(0) => Progress::unknown(),
            Some(total) => {
                Progress::known(ratio_down(u128::from(self.completed), u128::from(total)))
                    .expect("validated counts produce normalized progress")
            }
        }
    }
}

/// The host's declared task status, never inferred from time, progress, or art.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

impl TaskStatus {
    /// Classifies the explicitly supplied status; performs no state transition.
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed | Self::Cancelled)
    }
}

/// A replacement value supplied by the host, with no transition machine.
///
/// Running at 100%, succeeded with unknown progress, and failed with partial
/// progress are all valid. The caller owns stable task identity, generations,
/// stale-update rejection, labels, and any policy for changing these values.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TaskSnapshot {
    status: TaskStatus,
    progress: Progress,
}

impl TaskSnapshot {
    pub const fn new(status: TaskStatus, progress: Progress) -> Self {
        Self { status, progress }
    }

    pub const fn status(self) -> TaskStatus {
        self.status
    }

    pub const fn progress(self) -> Progress {
        self.progress
    }
}

/// One caller-observed activity event. No source is acquired or polled.
///
/// `generation` identifies the currently attached source/operation. `sequence`
/// must strictly increase within that generation, starting at any value including
/// zero. `at` is a caller-supplied monotonic observation timestamp, not clip time.
/// Events may share a timestamp. `units` is a caller-defined impulse; zero is valid
/// and still advances ordering, while a large impulse saturates the visual cap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamEvent {
    pub generation: u64,
    pub sequence: u64,
    pub at: HostTime,
    pub units: u64,
}

/// A fixed-size, host-owned visual accumulator with exact linear decay.
///
/// Each accepted event adds `units / capacity` to activity, capped at one.
/// Activity decreases by `elapsed / decay` between events, down to zero. Thus a
/// full signal empties over `decay`, and a half-full one empties in half that time.
/// Arithmetic retains exact fractional credit in `u128`; observing it never
/// changes future results or loses fractional decay. Conversion to [`Level`]
/// rounds down. Work and storage are bounded independently of event history.
///
/// Only [`Self::record`] and [`Self::reset`] mutate this value. There is no queue,
/// history replay, background worker, completion signal, or task-state ownership.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamActivity {
    generation: u64,
    sequence: Option<u64>,
    anchor: HostTime,
    credit: u128,
    decay: Period,
    capacity: u64,
}

impl StreamActivity {
    /// Starts an empty signal at the supplied observation time.
    /// A zero capacity is invalid; [`Period`] already requires positive decay.
    pub fn new(
        generation: u64,
        at: HostTime,
        decay: Period,
        capacity: u64,
    ) -> Result<Self, AdapterError> {
        if capacity == 0 {
            return Err(AdapterError::ZeroCapacity);
        }
        Ok(Self {
            generation,
            sequence: None,
            anchor: at,
            credit: 0,
            decay,
            capacity,
        })
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub const fn last_sequence(&self) -> Option<u64> {
        self.sequence
    }

    pub const fn decay(&self) -> Period {
        self.decay
    }

    pub const fn capacity(&self) -> u64 {
        self.capacity
    }

    /// Accepts an event only for this generation, with a strictly newer sequence
    /// and a timestamp at or after the latest accepted event/reset.
    /// Every error leaves the entire accumulator unchanged.
    pub fn record(&mut self, event: StreamEvent) -> Result<(), AdapterError> {
        if event.generation != self.generation {
            return Err(AdapterError::WrongGeneration);
        }
        if self
            .sequence
            .is_some_and(|previous| event.sequence <= previous)
        {
            return Err(AdapterError::NonIncreasingSequence);
        }
        let credit = self.credit_at(event.at)?;
        let impulse = u128::from(event.units) * u128::from(self.decay.as_nanos());
        self.credit = credit.saturating_add(impulse).min(self.max_credit());
        self.anchor = event.at;
        self.sequence = Some(event.sequence);
        Ok(())
    }

    /// Reads a deterministic signal at an explicit host time, without mutation.
    /// Queries can be repeated or shuffled at/after the latest event/reset time.
    /// Earlier queries fail because this bounded adapter stores no history.
    /// A future read does not prevent a later event before that read time.
    pub fn level(&self, now: HostTime) -> Result<Level, AdapterError> {
        let credit = self.credit_at(now)?;
        Ok(Level::new(ratio_down(credit, self.max_credit()))
            .expect("bounded stream credit produces a normalized level"))
    }

    /// Clears activity and sequence history for an explicitly replaced source.
    /// Requires a strictly greater generation and a nondecreasing event/reset
    /// timestamp; errors leave all fields unchanged. Generation values never
    /// wrap or get reused. After `u64::MAX`, create a separately identified host
    /// lifecycle instead of routing old events to a reused generation.
    ///
    /// Old queued events must retain their old generation, so they are rejected
    /// even if their sequence or timestamp exceeds the replacement's values.
    pub fn reset(&mut self, generation: u64, at: HostTime) -> Result<(), AdapterError> {
        if generation <= self.generation {
            return Err(AdapterError::NonIncreasingGeneration);
        }
        self.credit_at(at)?;
        self.generation = generation;
        self.sequence = None;
        self.anchor = at;
        self.credit = 0;
        Ok(())
    }

    fn max_credit(&self) -> u128 {
        u128::from(self.capacity) * u128::from(self.decay.as_nanos())
    }

    fn credit_at(&self, now: HostTime) -> Result<u128, AdapterError> {
        let elapsed = now
            .as_nanos()
            .checked_sub(self.anchor.as_nanos())
            .ok_or(AdapterError::BackwardsTime)?;
        let spent = u128::from(elapsed) * u128::from(self.capacity);
        Ok(self.credit.saturating_sub(spent))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterError {
    CountExceedsTotal,
    ZeroCapacity,
    WrongGeneration,
    NonIncreasingSequence,
    BackwardsTime,
    NonIncreasingGeneration,
}

impl fmt::Display for AdapterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::CountExceedsTotal => "completed count exceeds the known total",
            Self::ZeroCapacity => "stream activity capacity must be positive",
            Self::WrongGeneration => "stream event belongs to a different generation",
            Self::NonIncreasingSequence => "stream event sequence must strictly increase",
            Self::BackwardsTime => "stream time precedes the latest accepted event or reset",
            Self::NonIncreasingGeneration => "replacement generation must strictly increase",
        })
    }
}

impl std::error::Error for AdapterError {}

// Greatest f64 <= numerator / denominator for 0 <= numerator <= denominator.
// Values are normal (the smallest positive ratio is 1 / u128::MAX). Binary long
// division emits the leading bit and 52 fraction bits without rounding up. At
// most 128 + 52 iterations; every remainder operation is overflow-safe.
fn ratio_down(numerator: u128, denominator: u128) -> f64 {
    debug_assert!(denominator != 0 && numerator <= denominator);
    if numerator == 0 {
        return 0.0;
    }
    if numerator == denominator {
        return 1.0;
    }
    let mut remainder = numerator;
    let mut places = 1_u64;
    while !fraction_bit(&mut remainder, denominator) {
        places += 1;
    }
    let mut significand = 1_u64;
    for _ in 0..52 {
        significand = (significand << 1) | u64::from(fraction_bit(&mut remainder, denominator));
    }
    f64::from_bits(((1023 - places) << 52) | (significand & ((1_u64 << 52) - 1)))
}

fn fraction_bit(remainder: &mut u128, denominator: u128) -> bool {
    let complement = denominator - *remainder;
    if *remainder >= complement {
        *remainder -= complement;
        true
    } else {
        *remainder *= 2;
        false
    }
}
