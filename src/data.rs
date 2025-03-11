use super::*;

pub use crate::n::{InterpDataND, InterpDataNDOwned, InterpDataNDViewed};
pub use crate::one::{InterpData1D, InterpData1DOwned, InterpData1DViewed};
pub use crate::three::{InterpData3D, InterpData3DOwned, InterpData3DViewed};
pub use crate::two::{InterpData2D, InterpData2DOwned, InterpData2DViewed};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound = "
        D: DataOwned,
        D::Elem: Serialize + DeserializeOwned,
        Dim<[usize; N]>: Serialize + DeserializeOwned,
        [ArrayBase<D, Ix1>; N]: Serialize + DeserializeOwned,
    ")
)]
pub struct InterpData<D, const N: usize>
where
    Dim<[Ix; N]>: Dimension,
    D: Data + RawDataClone + Clone,
    D::Elem: Num + PartialOrd + Copy + Debug,
{
    pub grid: [ArrayBase<D, Ix1>; N],
    pub values: ArrayBase<D, Dim<[Ix; N]>>,
}
pub type InterpDataViewed<T, const N: usize> = InterpData<ndarray::ViewRepr<T>, N>;
pub type InterpDataOwned<T, const N: usize> = InterpData<ndarray::ViewRepr<T>, N>;

impl<D, const N: usize> InterpData<D, N>
where
    Dim<[Ix; N]>: Dimension,
    D: Data + RawDataClone + Clone,
    D::Elem: Num + PartialOrd + Copy + Debug,
{
    pub fn validate(&self) -> Result<(), ValidateError> {
        for i in 0..N {
            let i_grid_len = self.grid[i].len();
            // Check that each grid dimension has elements
            // Indexing `grid` directly is okay because empty dimensions are caught at compilation
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
