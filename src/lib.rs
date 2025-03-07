//! The `ninterp` crate provides
//! [multivariate interpolation](https://en.wikipedia.org/wiki/Multivariate_interpolation#Regular_grid)
//! over rectilinear grids of any dimensionality.
//!
//! There are hard-coded interpolators for lower dimensionalities (up to N = 3) for better runtime performance.
//!
//! A variety of interpolation strategies are implemented and exposed in the `prelude` module.
//! Custom interpolation strategies can be defined in downstream crates.
//!
//! ```text
//! cargo add ninterp
//! ```
//!
//! #### Feature Flags
//! - `serde`: support for [`serde`](https://crates.io/crates/serde)
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
//! Also see the [`examples`](https://github.com/NREL/ninterp/tree/a26c77caeac9e4ba2c5e8a4dbd652ce00b5747f3/examples)
//! directory for advanced examples:
//! - Strategy dynamic dispatch: [`dynamic_strategy.rs`](https://github.com/NREL/ninterp/blob/a26c77caeac9e4ba2c5e8a4dbd652ce00b5747f3/examples/dynamic_strategy.rs)
//!
//!   By default, construction of interpolators uses *static dispatch*,
//!   meaning strategy concrete types are determined at compilation.
//!   This gives increased performance at the cost of runtime flexibility.
//!   To allow swapping strategies at runtime,
//!   use *dynamic dispatch* by providing a trait object `Box<dyn Strategy1D>`/etc. to the `new` method.
//!
//! - Interpolator dynamic dispatch using `Box<dyn Interpolator>`: [`dynamic_interpolator.rs`](https://github.com/NREL/ninterp/blob/46d8436c4ac389e778392a28048fb9e32a80b8e0/examples/dynamic_interpolator.rs)
//!
//! - Defining custom strategies: [`custom_strategy.rs`](https://github.com/NREL/ninterp/blob/46d8436c4ac389e778392a28048fb9e32a80b8e0/examples/custom_strategy.rs)
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
//! call [`Interpolator::validate`]
//! to rerun these checks.
//! 
//! To change the extrapolation setting, call `set_extrapolate`.
//!
//! To change the interpolation strategy,
//! supply a `Box<dyn Strategy1D>`/etc. in the new method,
//! and call `set_strategy`.
//!
//! ## Strategies
//! An interpolation strategy (e.g. [`Linear`], [`Nearest`], [`LeftNearest`], [`RightNearest`]) must be specified.
//! Not all interpolation strategies are implemented for every dimensionality.
//! [`Linear`] and [`Nearest`] are implemented for all dimensionalities.
//!
//! Custom strategies can be defined. See
//! [`examples/custom_strategy.rs`](https://github.com/NREL/ninterp/blob/a26c77caeac9e4ba2c5e8a4dbd652ce00b5747f3/examples/custom_strategy.rs)
//! for an example.
//!
//! ## Extrapolation
//! An [`Extrapolate`] setting must be provided in the `new` method.
//! This controls what happens when a point is beyond the range of supplied coordinates.
//! The following setttings are applicable for all interpolators:
//! - [`Extrapolate::Fill(f64)`](`Extrapolate::Fill`)
//! - [`Extrapolate::Clamp`]
//! - [`Extrapolate::Error`]
//!
//! [`Extrapolate::Enable`] is valid for [`Linear`] in all dimensionalities.
//!
//! If you are unsure which variant to choose, [`Extrapolate::Error`] is likely what you want.
//!
//! ## Interpolation
//! Interpolation is executed by calling [`Interpolator::interpolate`].
//! 
//! The length of the interpolant point slice must be equal to the interpolator dimensionality.
//! The interpolator dimensionality can be retrieved by calling [`Interpolator::ndim`].

/// The `prelude` module exposes:
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
    pub use crate::strategy::{LeftNearest, Linear, Nearest, RightNearest};
    pub use crate::Extrapolate;
    pub use crate::Interpolator;
}

pub mod error;
pub mod strategy;

pub mod n;
pub mod one;
pub mod three;
pub mod two;
pub mod zero;

pub mod interpolator {
    pub use crate::n::InterpND;
    pub use crate::one::Interp1D;
    pub use crate::three::Interp3D;
    pub use crate::two::Interp2D;
    pub use crate::zero::Interp0D;
}

pub(crate) use error::*;
pub(crate) use strategy::*;

#[cfg(feature = "serde")]
pub(crate) use serde::{Deserialize, Serialize};

pub trait Interpolator {
    /// Interpolator dimensionality
    fn ndim(&self) -> usize;
    /// Validate interpolator data.
    fn validate(&self) -> Result<(), ValidateError>;
    /// Interpolate at supplied point.
    fn interpolate(&self, point: &[f64]) -> Result<f64, InterpolateError>;
    /// Get [`Extrapolate`] variant.
    ///
    /// This does not perform extrapolation.
    /// Instead, call [`Interpolator::interpolate`] on an instance using [`Extrapolate::Enable`].
    fn extrapolate(&self) -> Option<Extrapolate>;
    /// Set [`Extrapolate`] variant, checking validity.
    fn set_extrapolate(&mut self, extrapolate: Extrapolate) -> Result<(), ValidateError>;
}

impl Interpolator for Box<dyn Interpolator> {
    fn ndim(&self) -> usize {
        (**self).ndim()
    }
    fn validate(&self) -> Result<(), ValidateError> {
        (**self).validate()
    }
    fn interpolate(&self, point: &[f64]) -> Result<f64, InterpolateError> {
        (**self).interpolate(point)
    }
    fn extrapolate(&self) -> Option<Extrapolate> {
        (**self).extrapolate()
    }
    fn set_extrapolate(&mut self, extrapolate: Extrapolate) -> Result<(), ValidateError> {
        (**self).set_extrapolate(extrapolate)
    }
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
    /// If point is beyond grid limits, return this value instead.
    Fill(f64),
    /// Restrict interpolant point to the limits of the interpolation grid, using [`f64::clamp`].
    Clamp,
    /// Return an error when interpolant point is beyond the limits of the interpolation grid.
    #[default]
    Error,
}
