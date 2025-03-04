//! Crate error types

use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error(transparent)]
    ValidateError(#[from] ValidateError),
    #[error(transparent)]
    InterpolateError(#[from] InterpolateError),
    #[error("{0:?} field does not exist for interpolator variant")]
    NoSuchField(&'static str),
    #[error("{0}")]
    Other(String),
}

/// Error types that occur from a `validate()` call, before calling `interpolate()`
#[derive(Error, Debug, Clone)]
pub enum ValidateError {
    #[error(
        "selected `Extrapolate` variant ({0:?}) is unimplemented/inapplicable for interpolator"
    )]
    ExtrapolateSelection(crate::Extrapolate),
    #[error("supplied grid coordinates cannot be empty: dim {0}")]
    EmptyGrid(String),
    #[error("supplied coordinates must be sorted and non-repeating: dim {0}")]
    Monotonicity(String),
    #[error("supplied grid and values are not compatible shapes: dim {0}")]
    IncompatibleShapes(String),
    #[error("{0}")]
    Other(String),
}

#[derive(Error, Debug, Clone)]
pub enum InterpolateError {
    #[error("attempted to interpolate at point beyond grid data: {0}")]
    ExtrapolateError(String),
    #[error("surrounding values cannot be NaN: {0}")]
    NaNError(String),
    #[error("supplied point is invalid for interpolator: {0}")]
    InvalidPoint(String),
    #[error("{0}")]
    Other(String),
}
