//! Pre-defined interpolation strategies and traits for custom strategies

use super::*;
use std::fmt::Debug;

pub use crate::one::InterpData1D;
pub use crate::two::InterpData2D;
pub use crate::three::InterpData3D;
pub use crate::n::InterpDataND;

pub trait Strategy1D: Debug {
    fn interpolate(&self, data: &InterpData1D, point: &[f64; 1]) -> Result<f64, InterpolateError>;
    /// Does this type's [`Strategy1D::interpolate`] provision for extrapolation?
    fn allow_extrapolate(&self) -> bool;
}

impl Strategy1D for Box<dyn Strategy1D> {
    fn interpolate(&self, data: &InterpData1D, point: &[f64; 1]) -> Result<f64, InterpolateError> {
        (**self).interpolate(data, point)
    }
    fn allow_extrapolate(&self) -> bool {
        (**self).allow_extrapolate()
    }
}

pub trait Strategy2D: Debug {
    fn interpolate(&self, data: &InterpData2D, point: &[f64; 2]) -> Result<f64, InterpolateError>;
    /// Does this type's [`Strategy2D::interpolate`] provision for extrapolation?
    fn allow_extrapolate(&self) -> bool;
}

impl Strategy2D for Box<dyn Strategy2D> {
    fn interpolate(&self, data: &InterpData2D, point: &[f64; 2]) -> Result<f64, InterpolateError> {
        (**self).interpolate(data, point)
    }
    fn allow_extrapolate(&self) -> bool {
        (**self).allow_extrapolate()
    }
}

pub trait Strategy3D: Debug {
    fn interpolate(&self, data: &InterpData3D, point: &[f64; 3]) -> Result<f64, InterpolateError>;
    /// Does this type's [`Strategy3D::interpolate`] provision for extrapolation?
    fn allow_extrapolate(&self) -> bool;
}

impl Strategy3D for Box<dyn Strategy3D> {
    fn interpolate(&self, data: &InterpData3D, point: &[f64; 3]) -> Result<f64, InterpolateError> {
        (**self).interpolate(data, point)
    }
    fn allow_extrapolate(&self) -> bool {
        (**self).allow_extrapolate()
    }
}

pub trait StrategyND: Debug {
    fn interpolate(&self, data: &InterpDataND, point: &[f64]) -> Result<f64, InterpolateError>;
    /// Does this type's [`StrategyND::interpolate`] provision for extrapolation?
    fn allow_extrapolate(&self) -> bool;
}

impl StrategyND for Box<dyn StrategyND> {
    fn interpolate(&self, data: &InterpDataND, point: &[f64]) -> Result<f64, InterpolateError> {
        (**self).interpolate(data, point)
    }
    fn allow_extrapolate(&self) -> bool {
        (**self).allow_extrapolate()
    }
}

// This method contains code from RouteE Compass, another open-source NREL-developed tool
// <https://www.nrel.gov/transportation/route-energy-prediction-model.html>
// <https://github.com/NREL/routee-compass/>
pub fn find_nearest_index(arr: ArrayView1<f64>, target: f64) -> usize {
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
