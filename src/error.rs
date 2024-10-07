use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    ValidationError(#[from] ValidationError),
    #[error(transparent)]
    InterpolationError(#[from] InterpolationError),
    #[error("No such field exists for interpolator variant")]
    NoSuchField,
    #[error("{0}")]
    Other(String),
}

/// Error types that occur from a `validate()` call, before calling interpolate()
#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Selected `Strategy` variant ({0}) is unimplemented for interpolator variant")]
    StrategySelection(String),
    #[error("Selected `Extrapolate` variant ({0}) is unimplemented for interpolator variant")]
    ExtrapolationSelection(String),
    #[error("Supplied grid coordinates cannot be empty: dim {0}")]
    EmptyGrid(String),
    #[error("Supplied coordinates must be sorted and non-repeating: dim {0}")]
    Monotonicity(String),
    #[error("Supplied grid and values are not compatible shapes: dim {0}")]
    IncompatibleShapes(String),
    #[error("{0}")]
    Other(String),
}

#[derive(Error, Debug)]
pub enum InterpolationError {
    #[error("Attempted to interpolate at point beyond grid data: {0}")]
    ExtrapolationError(String),
    #[error("Surrounding values cannot be NaN: {0}")]
    NaNError(String),
    #[error("Supplied point is invalid for interpolator: {0}")]
    InvalidPoint(String),
    #[error("{0}")]
    Other(String),
}
