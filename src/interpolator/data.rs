use super::*;

pub use n::{InterpDataND, InterpDataNDOwned, InterpDataNDViewed};
pub use one::{InterpData1D, InterpData1DOwned, InterpData1DViewed};
pub use three::{InterpData3D, InterpData3DOwned, InterpData3DViewed};
pub use two::{InterpData2D, InterpData2DOwned, InterpData2DViewed};

/// Interpolator data for interpolators of concrete dimensionality `const N: usize`.
///
/// See [`InterpDataND`] for the N-dimensional interpolator data struct.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound(
        serialize = "
            D::Elem: Serialize,
            Dim<[usize; N]>: Serialize,
            [ArrayBase<D, Ix1>; N]: Serialize,
        ",
        deserialize = "
            D: DataOwned,
            D::Elem: Deserialize<'de>,
            Dim<[usize; N]>: Deserialize<'de> + Dimension,
            [usize; N]: IntoDimension<Dim = Dim<[usize; N]>>,
            [ArrayBase<D, Ix1>; N]: Deserialize<'de>,
        "
    ))
)]
pub struct InterpData<D, const N: usize>
where
    Dim<[Ix; N]>: Dimension,
    D: Data + RawDataClone + Clone,
    D::Elem: PartialEq + Debug,
{
    /// Coordinate grid: an `N`-length array of 1-dimensional [`ArrayBase<D, Ix1>`].
    /// - 1-D: `[x]`
    /// - 2-D: `[x, y]`
    /// - 3-D: `[x, y, z]`
    pub grid: [ArrayBase<D, Ix1>; N],
    /// Function values at coordinates: a single `N`-dimensional [`ArrayBase`].
    #[cfg_attr(feature = "serde", serde(with = "serde_ndim"))]
    pub values: ArrayBase<D, Dim<[Ix; N]>>,
}
pub type InterpDataViewed<T, const N: usize> = InterpData<ndarray::ViewRepr<T>, N>;
pub type InterpDataOwned<T, const N: usize> = InterpData<ndarray::ViewRepr<T>, N>;

impl<D, const N: usize> PartialEq for InterpData<D, N>
where
    Dim<[Ix; N]>: Dimension,
    D: Data + RawDataClone + Clone,
    D::Elem: PartialEq + Debug,
    ArrayBase<D, Ix1>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.grid == other.grid && self.values == other.values
    }
}

impl<D, const N: usize> InterpData<D, N>
where
    Dim<[Ix; N]>: Dimension,
    D: Data + RawDataClone + Clone,
    D::Elem: PartialOrd + Debug,
{
    pub fn validate(&self) -> Result<(), ValidateError> {
        for i in 0..N {
            let i_grid_len = self.grid[i].len();
            // Check that each grid dimension has elements
            if i_grid_len == 0 {
                return Err(ValidateError::EmptyGrid(i));
            }
            // Check that grid points are monotonically increasing
            if !self.grid[i].windows(2).into_iter().all(|w| w[0] <= w[1]) {
                return Err(ValidateError::Monotonicity(i));
            }
            // Check that grid and values are compatible shapes
            if i_grid_len != self.values.shape()[i] {
                return Err(ValidateError::IncompatibleShapes(i));
            }
        }
        Ok(())
    }
}
