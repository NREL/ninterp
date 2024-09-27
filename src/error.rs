use thiserror::Error;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Selected `Strategy` variant is unimplemented for interpolator")]
    StrategySelection,
    #[error("Selected `Extrapolate` variant is unimplemented for interpolator")]
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
