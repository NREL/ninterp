use super::*;

pub trait InterpMethods {
    /// Validate interpolator data
    fn validate(&self) -> Result<(), ValidateError>;
    /// Interpolate at supplied point
    fn interpolate(&self, point: &[f64]) -> Result<f64, InterpolateError>;
}
