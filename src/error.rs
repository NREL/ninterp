//! Custom error types

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    ValidationError(#[from] ValidationError),
    #[error(transparent)]
    InterpolationError(#[from] InterpolationError),
    #[error("no such field exists for interpolator variant")]
    NoSuchField,
    #[error("{0}")]
    Other(String),
}

/// Error types that occur from a `validate()` call, before calling interpolate()
#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("selected `Strategy` variant ({0}) is unimplemented for interpolator variant")]
    StrategySelection(String),
    #[error("selected `Extrapolate` variant ({0}) is unimplemented for interpolator variant")]
    ExtrapolationSelection(String),
    #[error("supplied grid coordinates cannot be empty: dim {0}")]
    EmptyGrid(String),
    #[error("supplied coordinates must be sorted and non-repeating: dim {0}")]
    Monotonicity(String),
    #[error("supplied grid and values are not compatible shapes: dim {0}")]
    IncompatibleShapes(String),
    #[error("{0}")]
    Other(String),
}

#[derive(Error, Debug)]
pub enum InterpolationError {
    #[error("sttempted to interpolate at point beyond grid data: {0}")]
    ExtrapolationError(String),
    #[error("surrounding values cannot be NaN: {0}")]
    NaNError(String),
    #[error("supplied point is invalid for interpolator: {0}")]
    InvalidPoint(String),
    #[error("{0}")]
    Other(String),
}
