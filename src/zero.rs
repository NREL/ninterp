//! 0-dimensional interpolation

use super::*;

const N: usize = 0;

/// 0-D interpolator
pub struct Interp0D(pub f64);
impl Interp0D {
    pub fn new(value: f64) -> Self {
        Self(value)
    }
}
impl Interpolator for Interp0D {
    /// Returns `0`
    fn ndim(&self) -> usize {
        N
    }

    /// Returns `Ok(())`
    fn validate(&self) -> Result<(), ValidateError> {
        Ok(())
    }

    fn interpolate(&self, point: &[f64]) -> Result<f64, InterpolateError> {
        if !point.is_empty() {
            return Err(InterpolateError::PointLength(0));
        }
        Ok(self.0)
    }

    /// Returns `None`
    fn extrapolate(&self) -> Option<Extrapolate> {
        None
    }

    /// Returns `Ok(())`
    fn set_extrapolate(&mut self, _extrapolate: Extrapolate) -> Result<(), ValidateError> {
        Ok(())
    }
}
