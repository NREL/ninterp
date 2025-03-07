//! The `ninterp` crate provides
//! [multivariate interpolation](https://en.wikipedia.org/wiki/Multivariate_interpolation#Regular_grid)
//! over rectilinear grids of any dimensionality.
//!
//! There are hard-coded interpolators for lower dimensionalities (up to N = 3) for better runtime performance.
//!
//! A variety of interpolation strategies are implemented and exposed in the `prelude` module.
//! Custom interpolation strategies can be defined in downstream crates.
//!
//!
//! # Feature Flags
//! - `serde`: support for [`serde`](https://crates.io/crates/serde)
//!
//! # Getting Started
//! A prelude module has been defined: `use ninterp::prelude::*;`
//!
//! This exposes a variety of interpolators:
//! - [`Interp1`](`interpolator::Interp1D`)
//! - [`Interp2`](`interpolator::Interp2D`)
//! - [`Interp3`](`interpolator::Interp3D`)
//! - [`InterpN`](`interpolator::InterpND`)
//!
//! There is also a constant-value 'interpolator':
//! [`Interp0`](`interpolator::Interp0D`).
//! This is useful when working with a `Box<dyn Interpolator>`
//!
//! Instantiation is done by calling an interpolator's `new` method.
//! For dimensionality N ≥ 1, this executes a validation step, preventing runtime panics.
//! When manually editing interpolator data, call [`Interpolator::validate`] to rerun these checks.
//! Utilize `set_strategy` and `set_extrapolate` methods to change the strategy and extrapolate setting.
//!
//! ## Strategies
//! An interpolation strategy (e.g. [`Linear`], [`Nearest`], [`LeftNearest`], [`RightNearest`]) must be specified.
//! Not all interpolation strategies are implemented for every dimensionality.
//! [`Linear`] and [`Nearest`] are implemented for all dimensionalities.
//!
//! Custom strategies can be defined. See `examples/custom_strategy.rs` for an example.
//!
//! ## Extrapolation
//! An [`Extrapolate`] setting must be specified.
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
//! Interpolation is executed by calling [`interpolate`](`Interpolator::interpolate`).
//! The length of the interpolant point slice must be equal to the interpolator dimensionality.
//! The interpolator dimensionality can be retrieved by calling [`Interpolator::ndim`].
//!
//! ## Examples
//! See `new` method documentation:
//! - [`Interp0D::new`](`interpolator::Interp0D::new`)
//! - [`Interp1D::new`](`interpolator::Interp1D::new`)
//! - [`Interp2D::new`](`interpolator::Interp2D::new`)
//! - [`Interp3D::new`](`interpolator::Interp3D::new`)
//! - [`InterpND::new`](`interpolator::InterpND::new`)
//!
//! See the `examples` directory for advanced examples.
//!
//! # Strategy dynamic dispatch
//! By default, construction of interpolators uses *static dispatch*,
//! meaning strategy concrete types are determined at compilation.
//! This gives increased performance at the cost of runtime flexibility.
//! To enable swapping strategies after instantiation,
//! use *dynamic dispatch* by providing a trait object `Box<dyn Trait>` to the constructor.
//!
//! See `examples/dynamic_strategy.rs` for an example.

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
