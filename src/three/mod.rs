//! 3-dimensional interpolation

use super::*;

mod strategies;

const N: usize = 3;

/// Data for [`Interp3D`]
#[non_exhaustive]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Data3D {
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub z: Vec<f64>,
    pub f_xyz: Vec<Vec<Vec<f64>>>,
}

/// 3-D interpolator
#[non_exhaustive]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Interp3D<S: Strategy3D> {
    pub data: Data3D,
    pub strategy: S,
    #[cfg_attr(feature = "serde", serde(default))]
    pub extrapolate: Extrapolate,
}

impl<S: Strategy3D> Interp3D<S> {
    /// Instantiate three-dimensional interpolator.
    ///
    /// Applicable interpolation strategies:
    /// - [`Linear`]
    /// - [`Nearest`]
    ///
    /// [`Extrapolate::Enable`] is valid for [`Linear`]
    ///
    /// # Example:
    /// ```
    /// use ninterp::prelude::*;
    /// // f(x, y, z) = 0.2 * x + 0.2 * y + 0.2 * z
    /// let interp = Interp3D::new(
    ///     // x
    ///     vec![1., 2.], // x0, x1
    ///     // y
    ///     vec![1., 2.], // y0, y1
    ///     // z
    ///     vec![1., 2.], // z0, z1
    ///     // f(x, y, z)
    ///     vec![
    ///         vec![
    ///             vec![0.6, 0.8], // f(x0, y0, z0), f(x0, y0, z1)
    ///             vec![0.8, 1.0], // f(x0, y1, z0), f(x0, y1, z1)
    ///         ],
    ///         vec![
    ///             vec![0.8, 1.0], // f(x1, y0, z0), f(x1, y0, z1)
    ///             vec![1.0, 1.2], // f(x1, y1, z0), f(x1, y1, z1)
    ///         ],
    ///     ],
    ///     Linear,
    ///     Extrapolate::Error, // return an error when point is out of bounds
    /// )
    /// .unwrap();
    /// assert_eq!(interp.interpolate(&[1.5, 1.5, 1.5]).unwrap(), 0.9);
    /// // out of bounds point with `Extrapolate::Error` fails
    /// assert!(matches!(
    ///     interp.interpolate(&[2.5, 2.5, 2.5]).unwrap_err(),
    ///     ninterp::error::InterpolateError::ExtrapolateError(_)
    /// ));
    /// ```
    pub fn new(
        x: Vec<f64>,
        y: Vec<f64>,
        z: Vec<f64>,
        f_xyz: Vec<Vec<Vec<f64>>>,
        strategy: S,
        extrapolate: Extrapolate,
    ) -> Result<Self, ValidateError> {
        let interpolator = Self {
            data: Data3D { x, y, z, f_xyz },
            strategy,
            extrapolate,
        };
        interpolator.validate()?;
        Ok(interpolator)
    }

    fn check_extrapolate(&self, extrapolate: Extrapolate) -> Result<(), ValidateError> {
        // Check applicability of strategy and extrapolate setting
        if matches!(extrapolate, Extrapolate::Enable) && !self.strategy.allow_extrapolate() {
            return Err(ValidateError::ExtrapolateSelection(self.extrapolate));
        }
        // If using Extrapolate::Enable,
        // check that each grid dimension has at least two elements
        if matches!(self.extrapolate, Extrapolate::Enable)
            && (self.data.x.len() < 2 || self.data.y.len() < 2 || self.data.z.len() < 2)
        {
            return Err(ValidateError::Other(
                "at least 2 data points are required for extrapolation".into(),
            ));
        }
        Ok(())
    }
}

impl<S: Strategy3D> Interpolator for Interp3D<S> {
    /// Returns `3`
    fn ndim(&self) -> usize {
        N
    }

    fn validate(&self) -> Result<(), ValidateError> {
        self.check_extrapolate(self.extrapolate)?;

        let x_grid_len = self.data.x.len();
        let y_grid_len = self.data.y.len();
        let z_grid_len = self.data.z.len();

        // Check that each grid dimension has elements
        if x_grid_len == 0 {
            return Err(ValidateError::EmptyGrid("x".into()));
        }
        if y_grid_len == 0 {
            return Err(ValidateError::EmptyGrid("y".into()));
        }
        if z_grid_len == 0 {
            return Err(ValidateError::EmptyGrid("z".into()));
        }

        // Check that grid points are monotonically increasing
        if !self.data.x.windows(2).all(|w| w[0] <= w[1]) {
            return Err(ValidateError::Monotonicity("x".into()));
        }
        if !self.data.y.windows(2).all(|w| w[0] <= w[1]) {
            return Err(ValidateError::Monotonicity("y".into()));
        }
        if !self.data.z.windows(2).all(|w| w[0] <= w[1]) {
            return Err(ValidateError::Monotonicity("z".into()));
        }

        // Check that grid and values are compatible shapes
        if x_grid_len != self.data.f_xyz.len() {
            return Err(ValidateError::IncompatibleShapes("x".into()));
        }
        if !self
            .data
            .f_xyz
            .iter()
            .map(Vec::len)
            .all(|y_val_len| y_val_len == y_grid_len)
        {
            return Err(ValidateError::IncompatibleShapes("y".into()));
        }
        if !self
            .data
            .f_xyz
            .iter()
            .flat_map(|y_vals| y_vals.iter().map(Vec::len))
            .all(|z_val_len| z_val_len == z_grid_len)
        {
            return Err(ValidateError::IncompatibleShapes("z".into()));
        }

        Ok(())
    }

    fn interpolate(&self, point: &[f64]) -> Result<f64, InterpolateError> {
        let point: &[f64; N] = point
            .try_into()
            .map_err(|_| InterpolateError::PointLength(N))?;
        let grid = [&self.data.x, &self.data.y, &self.data.z];
        let grid_names = ["x", "y", "z"];
        let mut errors = Vec::new();
        for dim in 0..N {
            if !(grid[dim].first().unwrap()..=grid[dim].last().unwrap()).contains(&&point[dim]) {
                match self.extrapolate {
                    Extrapolate::Enable => {}
                    Extrapolate::Fill(value) => return Ok(value),
                    Extrapolate::Clamp => {
                        let clamped_point = &[
                            point[0]
                                .clamp(*self.data.x.first().unwrap(), *self.data.x.last().unwrap()),
                            point[1]
                                .clamp(*self.data.y.first().unwrap(), *self.data.y.last().unwrap()),
                            point[2]
                                .clamp(*self.data.z.first().unwrap(), *self.data.z.last().unwrap()),
                        ];
                        return self.strategy.interpolate(&self.data, clamped_point);
                    }
                    Extrapolate::Error => {
                        errors.push(format!(
                            "\n    point[{dim}] = {:?} is out of bounds for {}-grid = {:?}",
                            point[dim], grid_names[dim], grid[dim],
                        ));
                    }
                };
            }
        }
        if !errors.is_empty() {
            return Err(InterpolateError::ExtrapolateError(errors.join("")));
        }
        self.strategy.interpolate(&self.data, point)
    }

    fn extrapolate(&self) -> Option<Extrapolate> {
        Some(self.extrapolate)
    }

    fn set_extrapolate(&mut self, extrapolate: Extrapolate) -> Result<(), ValidateError> {
        self.check_extrapolate(extrapolate)?;
        self.extrapolate = extrapolate;
        Ok(())
    }
}

impl Interp3D<Box<dyn Strategy3D>> {
    pub fn set_strategy(&mut self, strategy: Box<dyn Strategy3D>) -> Result<(), ValidateError> {
        self.strategy = strategy;
        self.check_extrapolate(self.extrapolate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear() {
        let x = vec![0.05, 0.10, 0.15];
        let y = vec![0.10, 0.20, 0.30];
        let z = vec![0.20, 0.40, 0.60];
        let f_xyz = vec![
            vec![vec![0., 1., 2.], vec![3., 4., 5.], vec![6., 7., 8.]],
            vec![vec![9., 10., 11.], vec![12., 13., 14.], vec![15., 16., 17.]],
            vec![
                vec![18., 19., 20.],
                vec![21., 22., 23.],
                vec![24., 25., 26.],
            ],
        ];
        let interp = Interp3D::new(
            x.clone(),
            y.clone(),
            z.clone(),
            f_xyz.clone(),
            Linear,
            Extrapolate::Error,
        )
        .unwrap();
        // Check that interpolating at grid points just retrieves the value
        for (i, x_i) in x.iter().enumerate() {
            for (j, y_j) in y.iter().enumerate() {
                for (k, z_k) in z.iter().enumerate() {
                    assert_eq!(
                        interp.interpolate(&[*x_i, *y_j, *z_k]).unwrap(),
                        f_xyz[i][j][k]
                    );
                }
            }
        }
        assert_eq!(
            interp.interpolate(&[x[0], y[0], 0.3]).unwrap(),
            0.4999999999999999 // 0.5
        );
        assert_eq!(
            interp.interpolate(&[x[0], 0.15, z[0]]).unwrap(),
            1.4999999999999996 // 1.5
        );
        assert_eq!(
            interp.interpolate(&[x[0], 0.15, 0.3]).unwrap(),
            1.9999999999999996 // 2.0
        );
        assert_eq!(
            interp.interpolate(&[0.075, y[0], z[0]]).unwrap(),
            4.499999999999999 // 4.5
        );
        assert_eq!(
            interp.interpolate(&[0.075, y[0], 0.3]).unwrap(),
            4.999999999999999 // 5.0
        );
        assert_eq!(
            interp.interpolate(&[0.075, 0.15, z[0]]).unwrap(),
            5.999999999999998 // 6.0
        );
    }

    #[test]
    fn test_linear_extrapolation() {
        let interp = Interp3D::new(
            vec![0.05, 0.10, 0.15],
            vec![0.10, 0.20, 0.30],
            vec![0.20, 0.40, 0.60],
            vec![
                vec![vec![0., 1., 2.], vec![3., 4., 5.], vec![6., 7., 8.]],
                vec![vec![9., 10., 11.], vec![12., 13., 14.], vec![15., 16., 17.]],
                vec![
                    vec![18., 19., 20.],
                    vec![21., 22., 23.],
                    vec![24., 25., 26.],
                ],
            ],
            Linear,
            Extrapolate::Enable,
        )
        .unwrap();
        // below x, below y, below z
        assert_eq!(interp.interpolate(&[0.01, 0.06, 0.17]).unwrap(), -8.55);
        assert_eq!(
            interp.interpolate(&[0.02, 0.08, 0.19]).unwrap(),
            -6.050000000000001
        );
        // below x, below y, above z
        assert_eq!(
            interp.interpolate(&[0.01, 0.06, 0.63]).unwrap(),
            -6.249999999999999
        );
        assert_eq!(
            interp.interpolate(&[0.02, 0.08, 0.65]).unwrap(),
            -3.749999999999999
        );
        // below x, above y, below z
        assert_eq!(
            interp.interpolate(&[0.01, 0.33, 0.17]).unwrap(),
            -0.44999999999999785
        );
        assert_eq!(
            interp.interpolate(&[0.02, 0.36, 0.19]).unwrap(),
            2.3500000000000014
        );
        // below x, above y, above z
        assert_eq!(
            interp.interpolate(&[0.01, 0.33, 0.63]).unwrap(),
            1.8499999999999994
        );
        assert_eq!(
            interp.interpolate(&[0.02, 0.36, 0.65]).unwrap(),
            4.650000000000003
        );
        // above x, below y, below z
        assert_eq!(
            interp.interpolate(&[0.17, 0.06, 0.17]).unwrap(),
            20.250000000000004
        );
        assert_eq!(interp.interpolate(&[0.19, 0.08, 0.19]).unwrap(), 24.55);
        // above x, below y, above z
        assert_eq!(interp.interpolate(&[0.17, 0.06, 0.63]).unwrap(), 22.55);
        assert_eq!(
            interp.interpolate(&[0.19, 0.08, 0.65]).unwrap(),
            26.849999999999994
        );
        // above x, above y, below z
        assert_eq!(
            interp.interpolate(&[0.17, 0.33, 0.17]).unwrap(),
            28.349999999999998
        );
        assert_eq!(
            interp.interpolate(&[0.19, 0.36, 0.19]).unwrap(),
            32.949999999999996
        );
        // above x, above y, above z
        assert_eq!(
            interp.interpolate(&[0.17, 0.33, 0.63]).unwrap(),
            30.650000000000006
        );
        assert_eq!(interp.interpolate(&[0.19, 0.36, 0.65]).unwrap(), 35.25);
    }

    #[test]
    fn test_linear_offset() {
        let interp = Interp3D::new(
            vec![0., 1.],
            vec![0., 1.],
            vec![0., 1.],
            vec![
                vec![vec![0., 1.], vec![2., 3.]],
                vec![vec![4., 5.], vec![6., 7.]],
            ],
            Linear,
            Extrapolate::Error,
        )
        .unwrap();
        assert_eq!(
            interp.interpolate(&[0.25, 0.65, 0.9]).unwrap(),
            3.1999999999999997
        ); // 3.2
    }

    #[test]
    fn test_nearest() {
        let x = vec![0., 1.];
        let y = vec![0., 1.];
        let z = vec![0., 1.];
        let f_xyz = vec![
            vec![vec![0., 1.], vec![2., 3.]],
            vec![vec![4., 5.], vec![6., 7.]],
        ];
        let interp = Interp3D::new(
            x.clone(),
            y.clone(),
            z.clone(),
            f_xyz.clone(),
            Nearest,
            Extrapolate::Error,
        )
        .unwrap();
        // Check that interpolating at grid points just retrieves the value
        for (i, x_i) in x.iter().enumerate() {
            for (j, y_j) in y.iter().enumerate() {
                for (k, z_k) in z.iter().enumerate() {
                    assert_eq!(
                        interp.interpolate(&[*x_i, *y_j, *z_k]).unwrap(),
                        f_xyz[i][j][k]
                    );
                }
            }
        }
        assert_eq!(interp.interpolate(&[0., 0., 0.]).unwrap(), 0.);
        assert_eq!(interp.interpolate(&[0.25, 0.25, 0.25]).unwrap(), 0.);
        assert_eq!(interp.interpolate(&[0.25, 0.75, 0.25]).unwrap(), 2.);
        assert_eq!(interp.interpolate(&[0., 1., 0.]).unwrap(), 2.);
        assert_eq!(interp.interpolate(&[0.75, 0.25, 0.75]).unwrap(), 5.);
        assert_eq!(interp.interpolate(&[0.75, 0.75, 0.75]).unwrap(), 7.);
        assert_eq!(interp.interpolate(&[1., 1., 1.]).unwrap(), 7.);
    }

    #[test]
    fn test_extrapolate_inputs() {
        // Extrapolate::Extrapolate
        assert!(matches!(
            Interp3D::new(
                vec![0.1, 1.1],
                vec![0.2, 1.2],
                vec![0.3, 1.3],
                vec![
                    vec![vec![0., 1.], vec![2., 3.]],
                    vec![vec![4., 5.], vec![6., 7.]],
                ],
                Nearest,
                Extrapolate::Enable,
            )
            .unwrap_err(),
            ValidateError::ExtrapolateSelection(_)
        ));
        // Extrapolate::Error
        let interp = Interp3D::new(
            vec![0.1, 1.1],
            vec![0.2, 1.2],
            vec![0.3, 1.3],
            vec![
                vec![vec![0., 1.], vec![2., 3.]],
                vec![vec![4., 5.], vec![6., 7.]],
            ],
            Linear,
            Extrapolate::Error,
        )
        .unwrap();
        assert!(matches!(
            interp.interpolate(&[-1., -1., -1.]).unwrap_err(),
            InterpolateError::ExtrapolateError(_)
        ));
        assert!(matches!(
            interp.interpolate(&[2., 2., 2.]).unwrap_err(),
            InterpolateError::ExtrapolateError(_)
        ));
    }

    #[test]
    fn test_extrapolate_fill_value() {
        let interp = Interp3D::new(
            vec![0.1, 1.1],
            vec![0.2, 1.2],
            vec![0.3, 1.3],
            vec![
                vec![vec![0., 1.], vec![2., 3.]],
                vec![vec![4., 5.], vec![6., 7.]],
            ],
            Linear,
            Extrapolate::Fill(f64::NAN),
        )
        .unwrap();
        assert_eq!(
            interp.interpolate(&[0.4, 0.4, 0.4]).unwrap(),
            1.7000000000000002
        );
        assert_eq!(interp.interpolate(&[0.8, 0.8, 0.8]).unwrap(), 4.5);
        assert!(interp.interpolate(&[0., 0., 0.]).unwrap().is_nan());
        assert!(interp.interpolate(&[0., 0., 2.]).unwrap().is_nan());
        assert!(interp.interpolate(&[0., 2., 0.]).unwrap().is_nan());
        assert!(interp.interpolate(&[0., 2., 2.]).unwrap().is_nan());
        assert!(interp.interpolate(&[2., 0., 0.]).unwrap().is_nan());
        assert!(interp.interpolate(&[2., 0., 2.]).unwrap().is_nan());
        assert!(interp.interpolate(&[2., 2., 0.]).unwrap().is_nan());
        assert!(interp.interpolate(&[2., 2., 2.]).unwrap().is_nan());
    }

    #[test]
    fn test_extrapolate_clamp() {
        let interp = Interp3D::new(
            vec![0.1, 1.1],
            vec![0.2, 1.2],
            vec![0.3, 1.3],
            vec![
                vec![vec![0., 1.], vec![2., 3.]],
                vec![vec![4., 5.], vec![6., 7.]],
            ],
            Linear,
            Extrapolate::Clamp,
        )
        .unwrap();
        assert_eq!(interp.interpolate(&[-1., -1., -1.]).unwrap(), 0.);
        assert_eq!(interp.interpolate(&[2., 2., 2.]).unwrap(), 7.);
    }
}
