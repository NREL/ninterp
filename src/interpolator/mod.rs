use super::*;

mod n;
mod one;
mod three;
mod two;
mod zero;

pub mod data;
pub mod enums;

pub use n::{InterpND, InterpNDOwned, InterpNDViewed};
pub use one::{Interp1D, Interp1DOwned, Interp1DViewed};
pub use three::{Interp3D, Interp3DOwned, Interp3DViewed};
pub use two::{Interp2D, Interp2DOwned, Interp2DViewed};
pub use zero::Interp0D;

/// An interpolator of data type `T`
///
/// This trait is dyn-compatible, meaning you can use:
/// `Box<dyn Interpolator<_>>`
/// and swap the contained interpolator at runtime.
pub trait Interpolator<T>: DynClone {
    /// Interpolator dimensionality.
    fn ndim(&self) -> usize;
    /// Validate interpolator data.
    fn validate(&mut self) -> Result<(), ValidateError>;
    /// Interpolate at supplied point.
    fn interpolate(&self, point: &[T]) -> Result<T, InterpolateError>;
}

clone_trait_object!(<T> Interpolator<T>);

impl<T> Interpolator<T> for Box<dyn Interpolator<T>> {
    fn ndim(&self) -> usize {
        (**self).ndim()
    }
    fn validate(&mut self) -> Result<(), ValidateError> {
        (**self).validate()
    }
    fn interpolate(&self, point: &[T]) -> Result<T, InterpolateError> {
        (**self).interpolate(point)
    }
}

/// Extrapolation strategy
///
/// Controls what happens if supplied interpolant point
/// is outside the bounds of the interpolation grid.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum Extrapolate<T> {
    /// Evaluate beyond the limits of the interpolation grid.
    Enable,
    /// If point is beyond grid limits, return this value instead.
    Fill(T),
    /// Restrict interpolant point to the limits of the interpolation grid, using [`num_traits::clamp`].
    Clamp,
    /// Wrap around to other end of periodic data.
    /// Does NOT check that first and last values are equal.
    Wrap,
    /// Return an error when interpolant point is beyond the limits of the interpolation grid.
    #[default]
    Error,
}

macro_rules! extrapolate_impl {
    ($InterpType:ident, $Strategy:ident) => {
        impl<D, S> $InterpType<D, S>
        where
            D: Data + RawDataClone + Clone,
            D::Elem: PartialEq + Debug,
            S: $Strategy<D> + Clone,
        {
            /// Set [`Extrapolate`] variant, checking validity.
            pub fn set_extrapolate(
                &mut self,
                extrapolate: Extrapolate<D::Elem>,
            ) -> Result<(), ValidateError> {
                self.check_extrapolate(&extrapolate)?;
                self.extrapolate = extrapolate;
                Ok(())
            }

            pub fn check_extrapolate(
                &self,
                extrapolate: &Extrapolate<D::Elem>,
            ) -> Result<(), ValidateError> {
                // Check applicability of strategy and extrapolate setting
                if matches!(extrapolate, Extrapolate::Enable) && !self.strategy.allow_extrapolate()
                {
                    return Err(ValidateError::ExtrapolateSelection(format!(
                        "{:?}",
                        self.extrapolate
                    )));
                }
                // If using Extrapolate::Enable,
                // check that each grid dimension has at least two elements
                if matches!(self.extrapolate, Extrapolate::Enable) {
                    for (i, g) in self.data.grid.iter().enumerate() {
                        if g.len() < 2 {
                            return Err(ValidateError::Other(format!(
                                "at least 2 data points are required for extrapolation: dim {i}",
                            )));
                        }
                    }
                }
                Ok(())
            }
        }
    };
}
pub(crate) use extrapolate_impl;
