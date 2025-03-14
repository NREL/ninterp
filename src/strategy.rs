//! Pre-defined interpolation strategies and traits for custom strategies

use super::*;

pub trait Strategy1D<D>: Debug + DynClone
where
    D: Data + RawDataClone + Clone,
    D::Elem: PartialEq + Debug,
{
    fn interpolate(
        &self,
        data: &InterpData1D<D>,
        point: &[D::Elem; 1],
    ) -> Result<D::Elem, InterpolateError>;
    /// Does this type's [`Strategy1D::interpolate`] provision for extrapolation?
    fn allow_extrapolate(&self) -> bool;
}

clone_trait_object!(<D> Strategy1D<D>);

impl<D> Strategy1D<D> for Box<dyn Strategy1D<D>>
where
    D: Data + RawDataClone + Clone,
    D::Elem: PartialEq + Debug,
{
    fn interpolate(
        &self,
        data: &InterpData1D<D>,
        point: &[D::Elem; 1],
    ) -> Result<D::Elem, InterpolateError> {
        (**self).interpolate(data, point)
    }
    fn allow_extrapolate(&self) -> bool {
        (**self).allow_extrapolate()
    }
}

pub trait Strategy2D<D>: Debug + DynClone
where
    D: Data + RawDataClone + Clone,
    D::Elem: PartialEq + Debug,
{
    fn interpolate(
        &self,
        data: &InterpData2D<D>,
        point: &[D::Elem; 2],
    ) -> Result<D::Elem, InterpolateError>;
    /// Does this type's [`Strategy2D::interpolate`] provision for extrapolation?
    fn allow_extrapolate(&self) -> bool;
}

clone_trait_object!(<D> Strategy2D<D>);

impl<D> Strategy2D<D> for Box<dyn Strategy2D<D>>
where
    D: Data + RawDataClone + Clone,
    D::Elem: PartialEq + Debug,
{
    fn interpolate(
        &self,
        data: &InterpData2D<D>,
        point: &[D::Elem; 2],
    ) -> Result<D::Elem, InterpolateError> {
        (**self).interpolate(data, point)
    }
    fn allow_extrapolate(&self) -> bool {
        (**self).allow_extrapolate()
    }
}

pub trait Strategy3D<D>: Debug + DynClone
where
    D: Data + RawDataClone + Clone,
    D::Elem: PartialEq + Debug,
{
    fn interpolate(
        &self,
        data: &InterpData3D<D>,
        point: &[D::Elem; 3],
    ) -> Result<D::Elem, InterpolateError>;
    /// Does this type's [`Strategy3D::interpolate`] provision for extrapolation?
    fn allow_extrapolate(&self) -> bool;
}

clone_trait_object!(<D> Strategy3D<D>);

impl<D> Strategy3D<D> for Box<dyn Strategy3D<D>>
where
    D: Data + RawDataClone + Clone,
    D::Elem: PartialEq + Debug,
{
    fn interpolate(
        &self,
        data: &InterpData3D<D>,
        point: &[D::Elem; 3],
    ) -> Result<D::Elem, InterpolateError> {
        (**self).interpolate(data, point)
    }
    fn allow_extrapolate(&self) -> bool {
        (**self).allow_extrapolate()
    }
}

pub trait StrategyND<D>: Debug + DynClone
where
    D: Data + RawDataClone + Clone,
    D::Elem: PartialEq + Debug,
{
    fn interpolate(
        &self,
        data: &InterpDataND<D>,
        point: &[D::Elem],
    ) -> Result<D::Elem, InterpolateError>;
    /// Does this type's [`StrategyND::interpolate`] provision for extrapolation?
    fn allow_extrapolate(&self) -> bool;
}

clone_trait_object!(<D> StrategyND<D>);

impl<D> StrategyND<D> for Box<dyn StrategyND<D>>
where
    D: Data + RawDataClone + Clone,
    D::Elem: PartialEq + Debug,
{
    fn interpolate(
        &self,
        data: &InterpDataND<D>,
        point: &[D::Elem],
    ) -> Result<D::Elem, InterpolateError> {
        (**self).interpolate(data, point)
    }
    fn allow_extrapolate(&self) -> bool {
        (**self).allow_extrapolate()
    }
}

// This method contains code from RouteE Compass, another open-source NREL-developed tool
// <https://www.nrel.gov/transportation/route-energy-prediction-model.html>
// <https://github.com/NREL/routee-compass/>
pub fn find_nearest_index<T: PartialOrd>(arr: ArrayView1<T>, target: &T) -> usize {
    if target == arr.last().unwrap() {
        return arr.len() - 2;
    }

    let mut low = 0;
    let mut high = arr.len() - 1;

    while low < high {
        let mid = low + (high - low) / 2;

        if &arr[mid] >= target {
            high = mid;
        } else {
            low = mid + 1;
        }
    }

    if low > 0 && &arr[low] >= target {
        low - 1
    } else {
        low
    }
}

/// Linear interpolation: <https://en.wikipedia.org/wiki/Linear_interpolation>
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Linear;

// TODO: `pub struct Quadratic;`
// Maybe `pub struct Polynomial(usize);` as well?
// with `pub type Quadratic = Polynomial(2)` and `pub type Cubic = Polynomial(3)`

/// Cubic spline interpolation: TODO
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Cubic;

/// Nearest value interpolation: <https://en.wikipedia.org/wiki/Nearest-neighbor_interpolation>
///
/// # Note
/// Float imprecision may affect the value returned near midpoints.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Nearest;

/// Left-nearest (previous value) interpolation: <https://en.wikipedia.org/wiki/Nearest-neighbor_interpolation>
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct LeftNearest;

/// Right-nearest (next value) interpolation: <https://en.wikipedia.org/wiki/Nearest-neighbor_interpolation>
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct RightNearest;
