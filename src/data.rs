use super::*;

#[derive(Debug)]
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
    D: Data,
    D::Elem: Copy + Debug,
{
    pub grid: [ArrayBase<D, Ix1>; N],
    pub values: ArrayBase<D, Dim<[Ix; N]>>,
}

impl<D, const N: usize> InterpData<D, N>
where
    Dim<[Ix; N]>: Dimension,
    D: Data,
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

// this doesn't compile, but implementing for each N does...
// #[cfg(feature = "uom")]
// #[allow(non_camel_case_types)]
// impl<'a, D, D_uom, U_uom, V_uom, const N: usize> InterpData<D, N>
// where
//     Dim<[Ix; N]>: Dimension,
//     D::Elem: Copy + Debug,
//     D: Data<Elem = uom::si::Quantity<D_uom, U_uom, V_uom>>,
//     D_uom: uom::si::Dimension + ?Sized + 'a,
//     U_uom: uom::si::Units<V_uom> + ?Sized + 'a,
//     V_uom: uom::num::Num + uom::Conversion<V_uom> + PartialOrd + Copy + Debug + 'a,
// {
//     /// Uses `std::mem::transmute` to get ArrayViews of `V` from ArrayViews of `Quantity<_, _, V>`.
//     /// Quantity is a repr(transparent) type.
//     pub fn values_view(&self) -> InterpData<ViewRepr<&V_uom>, N> {
//         unsafe {
//             InterpData {
//                 grid: core::array::from_fn(|i| std::mem::transmute(self.grid[i].view())),
//                 values: std::mem::transmute(self.values.view()),
//             }
//         }
//     }
// }

#[cfg(feature = "uom")]
#[allow(non_camel_case_types)]
impl<'a, D, D_uom, U_uom, V_uom> InterpData1D<D>
where
    D::Elem: Copy + Debug,
    D: Data<Elem = uom::si::Quantity<D_uom, U_uom, V_uom>>,
    D_uom: uom::si::Dimension + ?Sized + 'a,
    U_uom: uom::si::Units<V_uom> + ?Sized + 'a,
    V_uom: uom::num::Num + uom::Conversion<V_uom> + PartialOrd + Copy + Debug + 'a,
{
    /// Uses `std::mem::transmute` to get ArrayViews of `V` from ArrayViews of `Quantity<_, _, V>`.
    /// Quantity is a repr(transparent) type.
    pub fn values_view(&self) -> InterpData1D<ViewRepr<&V_uom>> {
        unsafe {
            InterpData1D {
                grid: core::array::from_fn(|i| std::mem::transmute(self.grid[i].view())),
                values: std::mem::transmute(self.values.view()),
            }
        }
    }
}

#[cfg(feature = "uom")]
#[allow(non_camel_case_types)]
impl<'a, D, D_uom, U_uom, V_uom> InterpData2D<D>
where
    D::Elem: Copy + Debug,
    D: Data<Elem = uom::si::Quantity<D_uom, U_uom, V_uom>>,
    D_uom: uom::si::Dimension + ?Sized + 'a,
    U_uom: uom::si::Units<V_uom> + ?Sized + 'a,
    V_uom: uom::num::Num + uom::Conversion<V_uom> + PartialOrd + Copy + Debug + 'a,
{
    /// Uses `std::mem::transmute` to get ArrayViews of `V` from ArrayViews of `Quantity<_, _, V>`.
    /// Quantity is a repr(transparent) type.
    pub fn values_view(&self) -> InterpData2D<ViewRepr<&V_uom>> {
        unsafe {
            InterpData2D {
                grid: core::array::from_fn(|i| std::mem::transmute(self.grid[i].view())),
                values: std::mem::transmute(self.values.view()),
            }
        }
    }
}

#[cfg(feature = "uom")]
#[allow(non_camel_case_types)]
impl<'a, D, D_uom, U_uom, V_uom> InterpData3D<D>
where
    D::Elem: Copy + Debug,
    D: Data<Elem = uom::si::Quantity<D_uom, U_uom, V_uom>>,
    D_uom: uom::si::Dimension + ?Sized + 'a,
    U_uom: uom::si::Units<V_uom> + ?Sized + 'a,
    V_uom: uom::num::Num + uom::Conversion<V_uom> + PartialOrd + Copy + Debug + 'a,
{
    /// Uses `std::mem::transmute` to get ArrayViews of `V` from ArrayViews of `Quantity<_, _, V>`.
    /// Quantity is a repr(transparent) type.
    pub fn values_view(&self) -> InterpData3D<ViewRepr<&V_uom>> {
        unsafe {
            InterpData3D {
                grid: core::array::from_fn(|i| std::mem::transmute(self.grid[i].view())),
                values: std::mem::transmute(self.values.view()),
            }
        }
    }
}

#[cfg(feature = "uom")]
#[allow(non_camel_case_types)]
impl<'a, D, D_uom, U_uom, V_uom> InterpDataND<D>
where
    D::Elem: Copy + Debug,
    D: Data<Elem = uom::si::Quantity<D_uom, U_uom, V_uom>>,
    D_uom: uom::si::Dimension + ?Sized + 'a,
    U_uom: uom::si::Units<V_uom> + ?Sized + 'a,
    V_uom: uom::num::Num + uom::Conversion<V_uom> + PartialOrd + Copy + Debug + 'a,
{
    /// Uses `std::mem::transmute` to get ArrayViews of `V` from ArrayViews of `Quantity<_, _, V>`.
    /// Quantity is a repr(transparent) type.
    pub fn values_view(&self) -> InterpDataND<ViewRepr<&V_uom>> {
        unsafe {
            InterpDataND {
                grid: self
                    .grid
                    .iter()
                    .map(|g| std::mem::transmute(g.view()))
                    .collect(),
                values: std::mem::transmute(self.values.view()),
            }
        }
    }
}

impl<'a, D, const N: usize> InterpData<D, N>
where
    Dim<[Ix; N]>: Dimension,
    D: Data,
    D::Elem: Copy + Debug,
{
    pub fn view(&self) -> InterpData<ViewRepr<&D::Elem>, N> {
        InterpData {
            grid: core::array::from_fn(|i| self.grid[i].view()),
            values: self.values.view(),
        }
    }
}

impl<'a, D> InterpDataND<D>
where
    D: Data,
    D::Elem: Copy + Debug,
{
    pub fn view(&self) -> InterpDataND<ViewRepr<&D::Elem>> {
        InterpDataND {
            grid: self.grid.iter().map(|g| g.view()).collect(),
            values: self.values.view(),
        }
    }
}
