//! 0-dimensional interpolation

use super::*;

const N: usize = 0;

/// 0-D interpolator
pub struct Interp0D<T>(pub T);
impl<T> Interp0D<T>
where
    T: Num + PartialOrd + Copy + Debug,
{
    /// Instantiate constant-value 'interpolator'.
    ///
    /// # Example:
    /// ```
    /// use ninterp::prelude::*;
    /// let const_value = 0.5;
    /// let interp = Interp0D::new(const_value);
    /// assert_eq!(
    ///     interp.interpolate(&[]).unwrap(), // an empty point is required for 0-D
    ///     const_value
    /// );
    /// ```
    pub fn new(value: T) -> Self {
        Self(value)
    }
}
impl<T> Interpolator<T> for Interp0D<T>
where
    T: Copy + Debug,
{
    /// Returns `0`
    fn ndim(&self) -> usize {
        N
    }

    /// Returns `Ok(())`
    fn validate(&self) -> Result<(), ValidateError> {
        Ok(())
    }

    fn interpolate(&self, point: &[T]) -> Result<T, InterpolateError> {
        if !point.is_empty() {
            return Err(InterpolateError::PointLength(N));
        }
        Ok(self.0)
    }

    /// Returns `None`
    fn extrapolate(&self) -> Option<Extrapolate<T>> {
        None
    }

    /// Returns `Ok(())`
    fn set_extrapolate(&mut self, _extrapolate: Extrapolate<T>) -> Result<(), ValidateError> {
        Ok(())
    }
}
