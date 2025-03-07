use super::*;

/// Interpolator data where N is known at compilation
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct InterpData<const N: usize>
where
    Dim<[Ix; N]>: Dimension,
{
    // #[cfg_attr(feature = "serde", serde(with = "serde_arrays"))]
    pub grid: [Array1<f64>; N],
    pub values: Array<f64, Dim<[Ix; N]>>,
}

pub type InterpData1D = InterpData<1>;
pub type InterpData2D = InterpData<2>;
pub type InterpData3D = InterpData<3>;

impl<const N: usize> InterpData<N>
where
    Dim<[Ix; N]>: Dimension,
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

impl InterpData1D {
    pub fn new(x: Array1<f64>, f_x: Array1<f64>) -> Result<Self, ValidateError> {
        let data = Self {
            grid: [x],
            values: f_x,
        };
        data.validate()?;
        Ok(data)
    }
}

impl InterpData2D {
    pub fn new(x: Array1<f64>, y: Array1<f64>, f_xy: Array2<f64>) -> Result<Self, ValidateError> {
        let data = Self {
            grid: [x, y],
            values: f_xy,
        };
        data.validate()?;
        Ok(data)
    }
}

impl InterpData3D {
    pub fn new(
        x: Array1<f64>,
        y: Array1<f64>,
        z: Array1<f64>,
        f_xyz: Array3<f64>,
    ) -> Result<Self, ValidateError> {
        let data = Self {
            grid: [x, y, z],
            values: f_xyz,
        };
        data.validate()?;
        Ok(data)
    }
}

/// Interpolator data where N is determined at runtime
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct InterpDataND {
    pub grid: Vec<Array1<f64>>,
    pub values: ArrayD<f64>,
}
impl InterpDataND {
    pub fn new(grid: Vec<Array1<f64>>, values: ArrayD<f64>) -> Result<Self, ValidateError> {
        let data = Self { grid, values };
        data.validate()?;
        Ok(data)
    }
}

impl InterpDataND {
    pub fn ndim(&self) -> usize {
        if self.values.len() == 1 {
            0
        } else {
            self.values.ndim()
        }
    }
    pub fn validate(&self) -> Result<(), ValidateError> {
        for i in 0..self.ndim() {
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
