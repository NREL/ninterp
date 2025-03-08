//! Pre-defined interpolation strategies and traits for custom strategies

use super::*;

pub use crate::n::InterpDataND;
pub use crate::one::InterpData1D;
pub use crate::three::InterpData3D;
pub use crate::two::InterpData2D;

pub trait Strategy1D<T>: Debug {
    fn interpolate(&self, data: &InterpData1D<T>, point: &[T; 1]) -> Result<T, InterpolateError>;
    /// Does this type's [`Strategy1D::interpolate`] provision for extrapolation?
    fn allow_extrapolate(&self) -> bool;
}

impl<T> Strategy1D<T> for Box<dyn Strategy1D<T>>
where
    T: Num + PartialOrd + Copy + Debug,
{
    fn interpolate(&self, data: &InterpData1D<T>, point: &[T; 1]) -> Result<T, InterpolateError> {
        (**self).interpolate(data, point)
    }
    fn allow_extrapolate(&self) -> bool {
        (**self).allow_extrapolate()
    }
}

pub trait Strategy2D<T>: Debug {
    fn interpolate(&self, data: &InterpData2D<T>, point: &[T; 2]) -> Result<T, InterpolateError>;
    /// Does this type's [`Strategy2D::interpolate`] provision for extrapolation?
    fn allow_extrapolate(&self) -> bool;
}

impl<T> Strategy2D<T> for Box<dyn Strategy2D<T>>
where
    T: Num + PartialOrd + Copy + Debug,
{
    fn interpolate(&self, data: &InterpData2D<T>, point: &[T; 2]) -> Result<T, InterpolateError> {
        (**self).interpolate(data, point)
    }
    fn allow_extrapolate(&self) -> bool {
        (**self).allow_extrapolate()
    }
}

pub trait Strategy3D<T>: Debug
where
    T: Num + PartialOrd + Copy + Debug,
{
    fn interpolate(&self, data: &InterpData3D<T>, point: &[T; 3]) -> Result<T, InterpolateError>;
    /// Does this type's [`Strategy3D::interpolate`] provision for extrapolation?
    fn allow_extrapolate(&self) -> bool;
}

impl<T> Strategy3D<T> for Box<dyn Strategy3D<T>>
where
    T: Num + PartialOrd + Copy + Debug,
{
    fn interpolate(&self, data: &InterpData3D<T>, point: &[T; 3]) -> Result<T, InterpolateError> {
        (**self).interpolate(data, point)
    }
    fn allow_extrapolate(&self) -> bool {
        (**self).allow_extrapolate()
    }
}

pub trait StrategyND<T>: Debug
where
    T: Num + PartialOrd + Copy + Debug,
{
    fn interpolate(&self, data: &InterpDataND<T>, point: &[T]) -> Result<T, InterpolateError>;
    /// Does this type's [`StrategyND::interpolate`] provision for extrapolation?
    fn allow_extrapolate(&self) -> bool;
}

impl<T> StrategyND<T> for Box<dyn StrategyND<T>>
where
    T: Num + PartialOrd + Copy + Debug,
{
    fn interpolate(&self, data: &InterpDataND<T>, point: &[T]) -> Result<T, InterpolateError> {
        (**self).interpolate(data, point)
    }
    fn allow_extrapolate(&self) -> bool {
        (**self).allow_extrapolate()
    }
}

// This method contains code from RouteE Compass, another open-source NREL-developed tool
// <https://www.nrel.gov/transportation/route-energy-prediction-model.html>
// <https://github.com/NREL/routee-compass/>
pub fn find_nearest_index<T: Num + PartialOrd + Copy + Debug>(
    arr: ArrayView1<T>,
    target: T,
) -> usize {
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
