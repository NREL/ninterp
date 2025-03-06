//! Pre-defined interpolation strategies and traits for custom strategies

use super::*;
use interpolator::*;
use std::fmt::Debug;

pub trait Interp1DStrategy: Debug {
    fn interpolate(
        &self,
        interpolator: &Interp1D,
        point: &[f64; 1],
    ) -> Result<f64, InterpolateError>;

    /// Does this type's [`Interp1DStrategy::interpolate`] provision for extrapolation?
    ///
    /// Used in [`Interp1D::validate`].
    fn allow_extrapolate(&self) -> bool;
}

pub trait Interp2DStrategy: Debug {
    fn interpolate(
        &self,
        interpolator: &Interp2D,
        point: &[f64; 2],
    ) -> Result<f64, InterpolateError>;

    /// Does this type's [`Interp2DStrategy::interpolate`] provision for extrapolation?
    ///
    /// Used in [`Interp2D::validate`].
    fn allow_extrapolate(&self) -> bool;
}

pub trait Interp3DStrategy: Debug {
    fn interpolate(
        &self,
        interpolator: &Interp3D,
        point: &[f64; 3],
    ) -> Result<f64, InterpolateError>;

    /// Does this type's [`Interp3DStrategy::interpolate`] provision for extrapolation?
    ///
    /// Used in [`Interp3D::validate`].
    fn allow_extrapolate(&self) -> bool;
}

pub trait InterpNDStrategy: Debug {
    fn interpolate(&self, interpolator: &InterpND, point: &[f64]) -> Result<f64, InterpolateError>;

    /// Does this type's [`InterpNDStrategy::interpolate`] provision for extrapolation?
    ///
    /// Used in [`InterpND::validate`].
    fn allow_extrapolate(&self) -> bool;
}

/// Linear interpolation: <https://en.wikipedia.org/wiki/Linear_interpolation>
#[derive(Debug)]
pub struct Linear;

/// Nearest value interpolation: <https://en.wikipedia.org/wiki/Nearest-neighbor_interpolation>
///
/// # Note
/// Float imprecision may affect the value returned near midpoints.
#[derive(Debug)]
pub struct Nearest;

/// Left-nearest (previous value) interpolation: <https://en.wikipedia.org/wiki/Nearest-neighbor_interpolation>
#[derive(Debug)]
pub struct LeftNearest;

/// Right-nearest (next value) interpolation: <https://en.wikipedia.org/wiki/Nearest-neighbor_interpolation>
#[derive(Debug)]
pub struct RightNearest;
