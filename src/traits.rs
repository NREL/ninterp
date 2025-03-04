use super::*;

pub trait InterpMethods {
    /// Validate interpolator data
    fn validate(&self) -> Result<(), ValidateError>;
    /// Interpolate at supplied point
    fn interpolate(&self, point: &[f64]) -> Result<f64, InterpolateError>;
}

/// Linear interpolation: <https://en.wikipedia.org/wiki/Linear_interpolation>
pub trait Linear {
    fn linear(&self, point: &[f64]) -> Result<f64, InterpolateError>;
}

/// Left-nearest (previous value) interpolation: <https://en.wikipedia.org/wiki/Nearest-neighbor_interpolation>
pub trait LeftNearest {
    fn left_nearest(&self, point: &[f64]) -> Result<f64, InterpolateError>;
}

/// Right-nearest (next value) interpolation: <https://en.wikipedia.org/wiki/Nearest-neighbor_interpolation>
pub trait RightNearest {
    fn right_nearest(&self, point: &[f64]) -> Result<f64, InterpolateError>;
}

/// Nearest value interpolation: <https://en.wikipedia.org/wiki/Nearest-neighbor_interpolation>
pub trait Nearest {
    fn nearest(&self, point: &[f64]) -> Result<f64, InterpolateError>;
}
