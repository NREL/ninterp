//! 3-dimensional interpolation

use super::*;

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub(crate) struct Interp3D {
    pub(crate) x: Vec<f64>,
    pub(crate) y: Vec<f64>,
    pub(crate) z: Vec<f64>,
    pub(crate) f_xyz: Vec<Vec<Vec<f64>>>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub(crate) strategy: Strategy,
    #[cfg_attr(feature = "serde", serde(default))]
    pub(crate) extrapolate: Extrapolate,
}

impl Linear for Interp3D {
    fn linear(&self, point: &[f64]) -> Result<f64, InterpolateError> {
        // Extrapolation is checked previously in Interpolator::interpolate,
        // meaning:
        // - point is within grid bounds, or
        // - point is clamped, or
        // - extrapolation is enabled
        let grid = [&self.x, &self.y, &self.z];
        let lowers: Vec<usize> = (0..3)
            .map(|dim| {
                if &point[dim] < grid[dim].first().unwrap() {
                    0
                } else if &point[dim] > grid[dim].last().unwrap() {
                    grid[dim].len() - 2
                } else {
                    find_nearest_index(grid[dim], point[dim])
                }
            })
            .collect();
        // x
        let x_l = lowers[0];
        let x_u = x_l + 1;
        let x_diff = (point[0] - self.x[x_l]) / (self.x[x_u] - self.x[x_l]);
        // y
        let y_l = lowers[1];
        let y_u = y_l + 1;
        let y_diff = (point[1] - self.y[y_l]) / (self.y[y_u] - self.y[y_l]);
        // z
        let z_l = lowers[2];
        let z_u = z_l + 1;
        let z_diff = (point[2] - self.z[z_l]) / (self.z[z_u] - self.z[z_l]);
        // interpolate in the x-direction
        let f00 = self.f_xyz[x_l][y_l][z_l] * (1.0 - x_diff) + self.f_xyz[x_u][y_l][z_l] * x_diff;
        let f01 = self.f_xyz[x_l][y_l][z_u] * (1.0 - x_diff) + self.f_xyz[x_u][y_l][z_u] * x_diff;
        let f10 = self.f_xyz[x_l][y_u][z_l] * (1.0 - x_diff) + self.f_xyz[x_u][y_u][z_l] * x_diff;
        let f11 = self.f_xyz[x_l][y_u][z_u] * (1.0 - x_diff) + self.f_xyz[x_u][y_u][z_u] * x_diff;
        // interpolate in the y-direction
        let f0 = f00 * (1.0 - y_diff) + f10 * y_diff;
        let f1 = f01 * (1.0 - y_diff) + f11 * y_diff;
        // interpolate in the z-direction
        Ok(f0 * (1.0 - z_diff) + f1 * z_diff)
    }
}

impl Nearest for Interp3D {
    fn nearest(&self, point: &[f64]) -> Result<f64, InterpolateError> {
        // x
        let x_l = find_nearest_index(&self.x, point[0]);
        let x_u = x_l + 1;
        let x_diff = (point[0] - self.x[x_l]) / (self.x[x_u] - self.x[x_l]);
        let i = if x_diff < 0.5 { x_l } else { x_u };
        // y
        let y_l = find_nearest_index(&self.y, point[1]);
        let y_u = y_l + 1;
        let y_diff = (point[1] - self.y[y_l]) / (self.y[y_u] - self.y[y_l]);
        let j = if y_diff < 0.5 { y_l } else { y_u };
        // z
        let z_l = find_nearest_index(&self.z, point[2]);
        let z_u = z_l + 1;
        let z_diff = (point[2] - self.z[z_l]) / (self.z[z_u] - self.z[z_l]);
        let k = if z_diff < 0.5 { z_l } else { z_u };

        Ok(self.f_xyz[i][j][k])
    }
}

impl InterpMethods for Interp3D {
    fn validate(&self) -> Result<(), ValidateError> {
        // Check applicablitity of strategy and extrapolate
        match (&self.strategy, &self.extrapolate) {
            // inapplicable strategies
            (Strategy::LeftNearest | Strategy::RightNearest, _) => {
                Err(ValidateError::StrategySelection(self.strategy))
            }
            // inapplicable combinations of strategy + extrapolate
            (Strategy::Nearest, Extrapolate::Enable) => {
                Err(ValidateError::ExtrapolateSelection(self.extrapolate))
            }
            _ => Ok(()),
        }?;

        let x_grid_len = self.x.len();
        let y_grid_len = self.y.len();
        let z_grid_len = self.z.len();

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

        // If using Extrapolate::Enable,
        // check that each grid dimension has at least two elements
        if matches!(self.extrapolate, Extrapolate::Enable)
            && (x_grid_len < 2 || y_grid_len < 2 || z_grid_len < 2)
        {
            return Err(ValidateError::Other(
                "at least 2 data points are required for extrapolation".into(),
            ));
        }

        // Check that grid points are monotonically increasing
        if !self.x.windows(2).all(|w| w[0] <= w[1]) {
            return Err(ValidateError::Monotonicity("x".into()));
        }
        if !self.y.windows(2).all(|w| w[0] <= w[1]) {
            return Err(ValidateError::Monotonicity("y".into()));
        }
        if !self.z.windows(2).all(|w| w[0] <= w[1]) {
            return Err(ValidateError::Monotonicity("z".into()));
        }

        // Check that grid and values are compatible shapes
        if x_grid_len != self.f_xyz.len() {
            return Err(ValidateError::IncompatibleShapes("x".into()));
        }
        if !self
            .f_xyz
            .iter()
            .map(Vec::len)
            .all(|y_val_len| y_val_len == y_grid_len)
        {
            return Err(ValidateError::IncompatibleShapes("y".into()));
        }
        if !self
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
        match self.strategy {
            Strategy::Linear => self.linear(point),
            Strategy::Nearest => self.nearest(point),
            _ => unreachable!(),
        }
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
        let interp = Interpolator::new_3d(
            x.clone(),
            y.clone(),
            z.clone(),
            f_xyz.clone(),
            Strategy::Linear,
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
        let interp = Interpolator::new_3d(
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
            Strategy::Linear,
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
        let interp = Interpolator::new_3d(
            vec![0., 1.],
            vec![0., 1.],
            vec![0., 1.],
            vec![
                vec![vec![0., 1.], vec![2., 3.]],
                vec![vec![4., 5.], vec![6., 7.]],
            ],
            Strategy::Linear,
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
        let interp = Interpolator::new_3d(
            x.clone(),
            y.clone(),
            z.clone(),
            f_xyz.clone(),
            Strategy::Nearest,
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
            Interpolator::new_3d(
                vec![0.1, 1.1],
                vec![0.2, 1.2],
                vec![0.3, 1.3],
                vec![
                    vec![vec![0., 1.], vec![2., 3.]],
                    vec![vec![4., 5.], vec![6., 7.]],
                ],
                Strategy::Nearest,
                Extrapolate::Enable,
            )
            .unwrap_err(),
            ValidateError::ExtrapolateSelection(_)
        ));
        // Extrapolate::Error
        let interp = Interpolator::new_3d(
            vec![0.1, 1.1],
            vec![0.2, 1.2],
            vec![0.3, 1.3],
            vec![
                vec![vec![0., 1.], vec![2., 3.]],
                vec![vec![4., 5.], vec![6., 7.]],
            ],
            Strategy::Linear,
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
        let interp = Interpolator::new_3d(
            vec![0.1, 1.1],
            vec![0.2, 1.2],
            vec![0.3, 1.3],
            vec![
                vec![vec![0., 1.], vec![2., 3.]],
                vec![vec![4., 5.], vec![6., 7.]],
            ],
            Strategy::Linear,
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
        let interp = Interpolator::new_3d(
            vec![0.1, 1.1],
            vec![0.2, 1.2],
            vec![0.3, 1.3],
            vec![
                vec![vec![0., 1.], vec![2., 3.]],
                vec![vec![4., 5.], vec![6., 7.]],
            ],
            Strategy::Linear,
            Extrapolate::Clamp,
        )
        .unwrap();
        assert_eq!(interp.interpolate(&[-1., -1., -1.]).unwrap(), 0.);
        assert_eq!(interp.interpolate(&[2., 2., 2.]).unwrap(), 7.);
    }
}
