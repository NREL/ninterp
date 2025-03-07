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
//! A prelude module has been defined: `use ninterp::prelude::*;`
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
//! - [`interpolator::Interp0D`]
//! - [`interpolator::Interp1D::new`]
//! - [`interpolator::Interp2D::new`]
//! - [`interpolator::Interp3D::new`]
//! - [`interpolator::InterpND::new`]
//!

/// The `prelude` module exposes:
/// - All interpolator structs:
///   - [`Interp0D`]
///   - [`Interp1D`]
///   - [`Interp2D`]
///   - [`Interp3D`]
///   - [`InterpND`]
/// - Their common trait: [`Interpolator`]
/// - Interpolation strategies:
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
