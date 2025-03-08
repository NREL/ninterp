//! Crate error types

use thiserror::Error;

/// Error in interpolator data validation
#[derive(Error, Debug, Clone)]
pub enum ValidateError {
    #[error("selected `Strategy` ({0}) is unimplemented/inapplicable for interpolator")]
    StrategySelection(&'static str),
    #[error("selected `Extrapolate` variant ({0}) is unimplemented/inapplicable for interpolator")]
    ExtrapolateSelection(String),
    #[error("supplied grid coordinates cannot be empty: dim {0}")]
    EmptyGrid(usize),
    #[error("supplied coordinates must be sorted and non-repeating: dim {0}")]
    Monotonicity(usize),
    #[error("supplied grid and values are not compatible shapes: dim {0}")]
    IncompatibleShapes(usize),
    #[error("{0}")]
    Other(String),
}

/// Error in interpolation call
#[derive(Error, Debug, Clone)]
pub enum InterpolateError {
    #[error("attempted to interpolate at point beyond grid data: {0}")]
    ExtrapolateError(String),
    #[error("supplied point slice should have length {0} for {0}-D interpolation")]
    PointLength(usize),
    #[error("{0}")]
    Other(String),
}
