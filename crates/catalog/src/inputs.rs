//! Bounded caller-owned data. These values never measure or fetch anything.
use std::fmt;

/// Known normalized progress or an explicitly indeterminate operation.
///
/// The payload is private so every known value is finite and in `0..=1`.
/// Each sample receives the current value; switching operations cannot reuse
/// an earlier target, smoothing state, or a completion event.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Progress(Option<f64>);
impl Progress {
    /// A known target. Rejects NaN, infinity, and values outside `0..=1`.
    pub fn known(value: f64) -> Result<Self, InputError> {
        validate(value).map(|value| Self(Some(value)))
    }
    pub const fn unknown() -> Self {
        Self(None)
    }
    pub const fn value(self) -> Option<f64> {
        self.0
    }
    /// Conservative whole-cell quantization of the exact supplied binary value.
    /// Integer arithmetic avoids rounding a just-below-boundary target upward.
    pub(crate) fn filled(self, cells: u16) -> Option<u16> {
        self.0.map(|value| {
            let bits = value.to_bits();
            let exponent = (bits >> 52) & 0x7ff;
            if exponent == 0 {
                return 0; // Zero/subnormal values cannot fill even one u16 cell.
            }
            let significand = (bits & ((1_u64 << 52) - 1)) | (1_u64 << 52);
            let shift = 1075 - exponent;
            if shift >= 128 {
                0
            } else {
                ((u128::from(significand) * u128::from(cells)) >> shift) as u16
            }
        })
    }
}
impl Default for Progress {
    fn default() -> Self {
        Self::unknown()
    }
}
impl TryFrom<f64> for Progress {
    type Error = InputError;
    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::known(value)
    }
}

/// Finite normalized caller-supplied telemetry in `0..=1`.
/// The host decides what this level measures and how often to update it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Level(f64);
impl Level {
    pub const ZERO: Self = Self(0.0);
    pub const MAX: Self = Self(1.0);
    /// Rejects NaN, infinity, and values outside `0..=1`; never silently clamps.
    pub fn new(value: f64) -> Result<Self, InputError> {
        validate(value).map(Self)
    }
    pub const fn value(self) -> f64 {
        self.0
    }
}
impl Default for Level {
    fn default() -> Self {
        Self::ZERO
    }
}
impl TryFrom<f64> for Level {
    type Error = InputError;
    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputError {
    NonFinite,
    OutOfRange,
}
impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NonFinite => "a normalized input must be finite",
            Self::OutOfRange => "a normalized input must be in 0..=1",
        })
    }
}
impl std::error::Error for InputError {}

fn validate(value: f64) -> Result<f64, InputError> {
    if !value.is_finite() {
        Err(InputError::NonFinite)
    } else if !(0.0..=1.0).contains(&value) {
        Err(InputError::OutOfRange)
    } else {
        Ok(value)
    }
}
