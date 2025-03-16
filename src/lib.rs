//! The `ninterp` crate provides
//! [multivariate interpolation](https://en.wikipedia.org/wiki/Multivariate_interpolation#Regular_grid)
//! over rectilinear grids of any dimensionality.
//!
//! There are hard-coded interpolators for lower dimensionalities (up to N = 3) for better runtime performance.
//! All interpolators work with both owned and borrowed arrays (array views) of various types.
//!
//! A variety of interpolation strategies are implemented and exposed in the `prelude` module.
//! Custom interpolation strategies can be defined in downstream crates.
//!
//! ```text
//! cargo add ninterp
//! ```
//!
//! ### Cargo Features
//! - `serde`: support for [`serde`](https://crates.io/crates/serde) ([caveat](https://github.com/NREL/ninterp/issues/5))
//!   ```text
//!   cargo add ninterp --features serde
//!   ```
//!
//! # Examples
//! See examples in `new` method documentation:
//! - [`Interp0D::new`](`interpolator::Interp0D::new`)
//! - [`Interp1D::new`](`interpolator::Interp1D::new`)
//! - [`Interp2D::new`](`interpolator::Interp2D::new`)
//! - [`Interp3D::new`](`interpolator::Interp3D::new`)
//! - [`InterpND::new`](`interpolator::InterpND::new`)
//!
//! Also see the `examples` directory for advanced examples:
//! - `dynamic_strategy.rs`
//!
//!   Strategy dynamic dispatch
//!
//!   By default, construction of interpolators uses *static dispatch*,
//!   meaning strategy concrete types are determined at compilation.
//!   This gives increased performance at the cost of runtime flexibility.
//!   To allow swapping strategies at runtime,
//!   use *dynamic dispatch* by providing a boxed trait object
//!   `Box<dyn Strategy1D>`/etc. to the `new` method.
//!
//! - `dynamic_interpolator.rs`
//!
//!   Interpolator dynamic dispatch using `Box<dyn Interpolator>`
//!
//! - `custom_strategy.rs`
//!
//!   Defining custom strategies
//!
//! - `uom.rs`
//!
//!   Using transmutable (transparent) types, such as `uom::si::Quantity`
//!
//! - Using transmutable (transparent) types, such as `uom::si::Quantity`: [`uom.rs`](https://github.com/NREL/ninterp/blob/de2c770dc3614ba43af9e015481fecdc20538380/examples/uom.rs)
//!
//! # Overview
//! A prelude module has been defined:
//! ```rust,text
//! use ninterp::prelude::*;
//! ```
//!
//! This exposes all strategies and a variety of interpolators:
//! - [`Interp1D`](`interpolator::Interp1D`)
//! - [`Interp2D`](`interpolator::Interp2D`)
//! - [`Interp3D`](`interpolator::Interp3D`)
//! - [`InterpND`](`interpolator::InterpND`)
//!
//! There is also a constant-value 'interpolator':
//! [`Interp0D`](`interpolator::Interp0D`).
//! This is useful when working with a `Box<dyn Interpolator>`
//!
//! Instantiation is done by calling an interpolator's `new` method.
//! For dimensionalities N ≥ 1, this executes a validation step, preventing runtime panics.
//! After editing interpolator data,
//! call the InterpData's `validate` method
//! or [`Interpolator::validate`]
//! to rerun these checks.
//!
//! To change the extrapolation setting, call `set_extrapolate`.
//!
//! To change the interpolation strategy,
//! supply a `Box<dyn Strategy1D>`/etc. upon instantiation,
//! and call `set_strategy`.
//!
//! ## Strategies
//! An interpolation strategy (e.g. [`Linear`], [`Cubic`], [`Nearest`], [`LeftNearest`], [`RightNearest`]) must be specified.
//! Not all interpolation strategies are implemented for every dimensionality.
//! [`Linear`] and [`Nearest`] are implemented for all dimensionalities.
//!
//! Custom strategies can be defined. See `examples/custom_strategy.rs` for an example.
//!
//! ## Extrapolation
//! An [`Extrapolate`] setting must be provided in the `new` method.
//! This controls what happens when a point is beyond the range of supplied coordinates.
//! The following settings are applicable for all interpolators:
//! - [`Extrapolate::Fill(T)`](`Extrapolate::Fill`)
//! - [`Extrapolate::Clamp`]
//! - [`Extrapolate::Error`]
//!
//! [`Extrapolate::Enable`] is valid for [`Linear`] for all dimensionalities.
//!
//! If you are unsure which variant to choose, [`Extrapolate::Error`] is likely what you want.
//!
//! ## Interpolation
//! Interpolation is executed by calling [`Interpolator::interpolate`].
//!
//! The length of the interpolant point slice must be equal to the interpolator dimensionality.
//! The interpolator dimensionality can be retrieved by calling [`Interpolator::ndim`].

/// The `prelude` module exposes a variety of types:
/// - All interpolator structs:
///   - [`Interp0D`](`interpolator::Interp0D`)
///   - [`Interp1D`](`interpolator::Interp1D`)
///   - [`Interp2D`](`interpolator::Interp2D`)
///   - [`Interp3D`](`interpolator::Interp3D`)
///   - [`InterpND`](`interpolator::InterpND`)
/// - Their common trait: [`Interpolator`]
/// - Pre-defined interpolation strategies:
///   - [`Linear`]
///   - [`Nearest`]
///   - [`LeftNearest`]
///   - [`RightNearest`]
/// - The extrapolation setting enum: [`Extrapolate`]
pub mod prelude {
    pub use crate::interpolator::*;
    pub use crate::strategy::{Cubic, LeftNearest, Linear, Nearest, RightNearest};
    pub use crate::Extrapolate;
    pub use crate::Interpolator;
}

pub mod data;
pub mod error;
pub mod strategy;

pub mod n;
pub mod one;
pub mod three;
pub mod two;
pub mod zero;

pub mod interpolator {
    pub use crate::n::{InterpND, InterpNDOwned, InterpNDViewed};
    pub use crate::one::{Interp1D, Interp1DOwned, Interp1DViewed};
    pub use crate::three::{Interp3D, Interp3DOwned, Interp3DViewed};
    pub use crate::two::{Interp2D, Interp2DOwned, Interp2DViewed};
    pub use crate::zero::Interp0D;
}

pub(crate) use data::*;
pub(crate) use error::*;
pub(crate) use strategy::*;

pub(crate) use std::fmt::Debug;

pub use ndarray;
pub(crate) use ndarray::prelude::*;
pub(crate) use ndarray::{Data, Ix, RawDataClone};

pub(crate) use num_traits::{clamp, Euclid, Float, Num, NumCast, One, Zero};

pub(crate) use dyn_clone::*;

#[cfg(feature = "serde")]
pub(crate) use ndarray::DataOwned;
#[cfg(feature = "serde")]
pub(crate) use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[cfg(test)]
/// Alias for [`approx::assert_abs_diff_eq`] with `epsilon = 1e-6`
macro_rules! assert_approx_eq {
    ($a:expr, $b:expr $(,)?) => {
        approx::assert_abs_diff_eq!($a, $b, epsilon = 1e-6)
    };
    ($a:expr, $b:expr, $eps:expr $(,)?) => {
        approx::assert_abs_diff_eq!($a, $b, epsilon = $eps)
    };
}
#[cfg(test)]
pub(crate) use assert_approx_eq;

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

/// Wrap value around data bounds.
/// Assumes `min` < `max`.
pub(crate) fn wrap<T: Num + Euclid + Copy>(input: T, min: T, max: T) -> T {
    min + (input - min).rem_euclid(&(max - min))
}

#[cfg(test)]
mod tests {
    use crate::wrap;

    #[test]
    fn test() {
        assert_eq!(wrap(-3, -2, 5), 4);
        assert_eq!(wrap(3, -2, 5), 3);
        assert_eq!(wrap(6, -2, 5), -1);
        assert_eq!(wrap(5, 0, 10), 5);
        assert_eq!(wrap(11, 0, 10), 1);
        assert_eq!(wrap(-3, 0, 10), 7);
        assert_eq!(wrap(-11, 0, 10), 9);
        assert_eq!(wrap(-0.1, -2., -1.), -1.1);
        assert_eq!(wrap(-0., -2., -1.), -2.0);
        assert_eq!(wrap(0.1, -2., -1.), -1.9);
        assert_eq!(wrap(-0.5, -1., 1.), -0.5);
        assert_eq!(wrap(0., -1., 1.), 0.);
        assert_eq!(wrap(0.5, -1., 1.), 0.5);
        assert_eq!(wrap(0.8, -1., 1.), 0.8);
    }
}
