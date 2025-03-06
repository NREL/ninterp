//! The `ninterp` crate provides
//! [multivariate interpolation](https://en.wikipedia.org/wiki/Multivariate_interpolation#Regular_grid)
//! over rectilinear grids of any dimensionality.
//! A variety of interpolation strategies are implemented, and more are likely to be added.
//!
//! There are hard-coded interpolators for lower dimensionalities (up to N = 3) for better runtime performance.
//!
//! # Feature Flags
//! - `serde`: support for [`serde`](https://crates.io/crates/serde)
//!
//! # Getting Started
//! A prelude module has been defined: `use ninterp::prelude::*;`.
//! This exposes the types necessary for usage:
//! - The main type: [`Interpolator`]
//! - Interpolation strategies:
//!   - [`Linear`]
//!   - [`Nearest`]
//!   - [`LeftNearest`]
//!   - [`RightNearest`]
//! - The extrapolation setting: [`Extrapolate`]
//!
//! Interpolation is executed by calling [`Interpolator::interpolate`].
//! The length of the supplied point slice must be equal to the interpolator dimensionality.
//!
//! For interpolators of dimensionality N ≥ 1:
//! - Instantiation is done via the Interpolator enum's `new_*` methods (`new_1d`, `new_2d`, `new_3d`, `new_nd`).
//!   These methods run a validation step that catches any potential errors early, preventing runtime panics.
//!   - To set or get field values, use the corresponding named methods (`x`, `set_x`, etc.).
//! - An interpolation [`Strategy`] (e.g. linear, left-nearest, etc.) must be specified.
//!   Not all interpolation strategies are implemented for every dimensionality.
//!   [`Linear`] and [`Nearest`] are implemented for all dimensionalities.
//! - An [`Extrapolate`] setting must be specified.
//!   This controls what happens when a point is beyond the range of supplied coordinates.
//!   If you are unsure which variant to choose, [`Extrapolate::Error`] is likely what you want.
//!   Linear extrapolation is implemented for all dimensionalities.
//!
//! For 0-D (constant-value) interpolators, instantiate directly, e.g. `Interp0D(0.5)`
//!
//! ## Examples
//! - [`Interp0D`]
//! - [`Interp1D::new`]
//! - [`Interp2D::new`]
//! - [`Interp3D::new`]
//! - [`InterpND::new`]
//!

pub mod prelude {
    pub use crate::Interpolator;
    pub use crate::interpolator::*;
    pub use crate::strategy::{LeftNearest, Linear, Nearest, RightNearest};
    pub use crate::Extrapolate;
}

pub mod error;
pub mod strategy;

mod n;
mod one;
mod three;
mod two;

pub mod interpolator {
    use super::*;

    pub struct Interp0D(pub f64);
    impl Interpolator for Interp0D {
        fn ndim(&self) -> usize {
            0
        }

        fn validate(&self) -> Result<(), ValidateError> {
            Ok(())
        }

        fn interpolate(&self, point: &[f64]) -> Result<f64, InterpolateError> {
            if !point.is_empty() {
                return Err(InterpolateError::PointLength(0));
            }
            Ok(self.0)
        }
    }

    pub use one::Interp1D;

    pub use two::Interp2D;

    pub use three::Interp3D;

    pub use n::InterpND;
}

// use
pub(crate) use error::*;
pub(crate) use strategy::*;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub trait Interpolator {
    fn ndim(&self) -> usize;
    /// Validate interpolator data
    fn validate(&self) -> Result<(), ValidateError>;
    /// Interpolate at supplied point
    fn interpolate(&self, point: &[f64]) -> Result<f64, InterpolateError>;
}

/// Extrapolation strategy
///
/// Controls what happens if supplied interpolant point
/// is outside the bounds of the interpolation grid.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum Extrapolate {
    /// Evaluate beyond the limits of the interpolation grid.
    Enable,
    /// Restrict interpolant point to the limits of the interpolation grid, using [`f64::clamp`].
    Clamp,
    /// If point is beyond grid limits, return this value instead.
    Fill(f64),
    /// Return an error when interpolant point is beyond the limits of the interpolation grid.
    #[default]
    Error,
}

// This method contains code from RouteE Compass, another open-source NREL-developed tool
// <https://www.nrel.gov/transportation/route-energy-prediction-model.html>
// <https://github.com/NREL/routee-compass/>
fn find_nearest_index(arr: &[f64], target: f64) -> usize {
    if &target == arr.last().unwrap() {
        return arr.len() - 2;
    }

    let mut low = 0;
    let mut high = arr.len() - 1;

    while low < high {
        let mid = low + (high - low) / 2;

        if arr[mid] >= target {
            high = mid;
        } else {
            low = mid + 1;
        }
    }

    if low > 0 && arr[low] >= target {
        low - 1
    } else {
        low
    }
}
