//! Pre-defined interpolation strategies and traits for custom strategies

use super::*;

pub trait Strategy1D<D>: Debug + DynClone
where
    D: Data + RawDataClone + Clone,
    D::Elem: PartialEq + Debug,
{
    fn init(&self, _data: &InterpData1D<D>) -> Result<(), ValidateError> {
        Ok(())
    }

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
    fn init(&self, data: &InterpData1D<D>) -> Result<(), ValidateError> {
        (**self).init(data)
    }

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
    fn init(&self, _data: &InterpData2D<D>) -> Result<(), ValidateError> {
        Ok(())
    }

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
    fn init(&self, data: &InterpData2D<D>) -> Result<(), ValidateError> {
        (**self).init(data)
    }

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
    fn init(&self, _data: &InterpData3D<D>) -> Result<(), ValidateError> {
        Ok(())
    }

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
    fn init(&self, data: &InterpData3D<D>) -> Result<(), ValidateError> {
        (**self).init(data)
    }

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
    fn init(&self, _data: &InterpDataND<D>) -> Result<(), ValidateError> {
        Ok(())
    }

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
    fn init(&self, data: &InterpDataND<D>) -> Result<(), ValidateError> {
        (**self).init(data)
    }

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

/// Cubic spline interpolation: TODO
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Cubic<T: Default> {
    pub boundary_cond: CubicBC<T>,
    pub coeffs: ArrayD<T>,
}

impl<T> Cubic<T>
where
    T: Default,
{
    pub fn new(bc: CubicBC<T>) -> Self {
        Self {
            boundary_cond: bc,
            coeffs: <ArrayD<T> as Default>::default(),
        }
    }

    pub fn natural() -> Self {
        Self::new(CubicBC::Natural)
    }

    pub fn clamped(a: T, b: T) -> Self {
        Self::new(CubicBC::Clamped(a, b))
    }

    pub fn not_a_knot() -> Self {
        Self::new(CubicBC::NotAKnot)
    }

    pub fn periodic() -> Self {
        Self::new(CubicBC::Periodic)
    }

    pub fn solve_coeffs(&mut self) {
        match &self.boundary_cond {
            CubicBC::Natural => {
                todo!()
            }
            _ => todo!(),
        }
    }
}

/// Cubic boundary conditions.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum CubicBC<T> {
    /// Second derivatives at endpoints are 0, thus extrapolation is linear.
    // https://www.math.ntnu.no/emner/TMA4215/2008h/cubicsplines.pdf
    #[default]
    Natural,
    /// Specific first derivatives at endpoints.
    Clamped(T, T),
    NotAKnot,
    // https://math.ou.edu/~npetrov/project-5093-s11.pdf
    Periodic,
}

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
