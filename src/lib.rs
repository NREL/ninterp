//! The `ninterp` crate provides
//! [multivariate interpolation](https://en.wikipedia.org/wiki/Multivariate_interpolation#Regular_grid)
//! over rectilinear grids of any dimensionality.
//!
//! There are hard-coded interpolators for lower dimensionalities (up to N = 3) for better runtime performance.
//! All interpolators work with both owned and borrowed arrays (array views) of various types.
//!
//! A variety of interpolation strategies are implemented and exposed in the [`prelude`] module.
//! Custom interpolation strategies can be defined in downstream crates.
//!
//! ```text
//! cargo add ninterp
//! ```
//!
//! ### Cargo Features
//! - `serde`: support for [`serde`](https://crates.io/crates/serde) 1.x
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
//! - **`dynamic_strategy.rs`**
//!
//!   Swapping strategies at runtime
//!   - Using strategy enums ([`strategy::enums::Strategy1DEnum`]/etc.)
//!     - Compatible with `serde`
//!     - Incompatible with custom strategies
//!   - Using [`Box<dyn Strategy1D>`]/etc. (dynamic dispatch)
//!     - Incompatible with `serde`
//!     - Compatible with custom strategies
//!     - Runtime cost
//!
//! - **`dynamic_interpolator.rs`**
//!
//!   Swapping interpolators at runtime
//!   - Using [`InterpolatorEnum`](interpolator::enums::InterpolatorEnum)
//!     - Compatible with `serde`
//!     - Incompatible with custom strategies
//!   - Using [`Box<dyn Interpolator>`] (dynamic dispatch)
//!     - Incompatible with `serde`
//!     - Compatible with custom strategies
//!     - Runtime cost
//!
//! - **`custom_strategy.rs`**
//!
//!   Defining custom strategies
//!
//! - **`uom.rs`**
//!
//!   Using transmutable (transparent) types, such as [`uom::si::Quantity`](https://docs.rs/uom/0.36.0/uom/si/struct.Quantity.html)
//!
//! # Overview
//! A [`prelude`] module has been defined:
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
//! This is useful when working with an [`InterpolatorEnum`](enums::InterpolatorEnum) or [`Box<dyn Interpolator>`]
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
//! supply a [`Strategy1DEnum`](strategy::enums::Strategy1DEnum)/etc. or [`Box<dyn Strategy1D>`]/etc. upon instantiation,
//! and call `set_strategy`.
//!
//! ## Strategies
//! An interpolation strategy (e.g.
//!   [`Linear`](strategy::Linear),
//!   [`Nearest`](strategy::Nearest),
//!   [`LeftNearest`](strategy::LeftNearest),
//!   [`RightNearest`](strategy::RightNearest))
//! must be specified.
//! Not all interpolation strategies are implemented for every dimensionality.
//! [`Linear`](strategy::Linear) and [`Nearest`](strategy::Nearest) are implemented for all dimensionalities.
//!
//! Custom strategies can be defined. See `examples/custom_strategy.rs` for an example.
//!
//! ## Extrapolation
//! An [`Extrapolate`] setting must be provided in the `new` method.
//! This controls what happens when a point is beyond the range of supplied coordinates.
//! The following settings are applicable for all interpolators:
//! - [`Extrapolate::Fill(T)`](`Extrapolate::Fill`)
//! - [`Extrapolate::Clamp`]
//! - [`Extrapolate::Wrap`]
//! - [`Extrapolate::Error`]
//!
//! [`Extrapolate::Enable`] is valid for [`Linear`](strategy::Linear) for all dimensionalities.
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
///   - A `serde`-compatible interpolator enum [`InterpolatorEnum`](`interpolator::enums::InterpolatorEnum`)
/// - Their common trait: [`Interpolator`]
/// - The [`strategy`] mod, containing pre-defined interpolation strategies:
///   - [`strategy::Linear`]
///   - [`strategy::Nearest`]
///   - [`strategy::LeftNearest`]
///   - [`strategy::RightNearest`]
///   - `serde`-compatible strategy enums: [`strategy::enums::Strategy1DEnum`]/etc.
/// - The extrapolation setting enum: [`Extrapolate`]
pub mod prelude {
    pub use crate::strategy;

    pub use crate::interpolator::{Extrapolate, Interpolator};

    pub use crate::interpolator::Interp0D;
    pub use crate::interpolator::{Interp1D, Interp1DOwned, Interp1DViewed};
    pub use crate::interpolator::{Interp2D, Interp2DOwned, Interp2DViewed};
    pub use crate::interpolator::{Interp3D, Interp3DOwned, Interp3DViewed};
    pub use crate::interpolator::{InterpND, InterpNDOwned, InterpNDViewed};

    pub use crate::interpolator::enums::{
        InterpolatorEnum, InterpolatorEnumOwned, InterpolatorEnumViewed,
    };
}

pub mod error;
pub mod strategy;

pub mod interpolator;
pub use interpolator::data;
pub(crate) use interpolator::data::*;
pub(crate) use interpolator::*;

pub(crate) use error::*;
pub(crate) use strategy::traits::*;

pub(crate) use std::fmt::Debug;

pub use ndarray;
pub(crate) use ndarray::prelude::*;
pub(crate) use ndarray::{Data, Ix, RawDataClone};

pub use num_traits;
pub(crate) use num_traits::{clamp, Euclid, Num, One};

pub(crate) use dyn_clone::*;

#[cfg(feature = "serde")]
pub(crate) use ndarray::DataOwned;
#[cfg(feature = "serde")]
pub(crate) use serde::{Deserialize, Serialize};

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

/// Wrap value around data bounds.
/// Assumes `min` < `max`.
pub(crate) fn wrap<T: Num + Euclid + Copy>(input: T, min: T, max: T) -> T {
    min + (input - min).rem_euclid(&(max - min))
}

#[cfg(test)]
mod tests {
    use super::wrap;

    #[test]
    fn test_wrap() {
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
