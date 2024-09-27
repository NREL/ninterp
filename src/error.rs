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
    #[error("Selected `Strategy` variant is unimplemented for interpolator variant")]
    StrategySelection,
    #[error("Selected `Extrapolate` variant is unimplemented for interpolator variant")]
    ExtrapolationSelection,
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
    #[error("Surrounding values cannot be NaN")]
    NaNError,
    #[error("Supplied point is invalid for interpolator: {0}")]
    InvalidPoint(String),
    #[error("{0}")]
    Other(String),
}
