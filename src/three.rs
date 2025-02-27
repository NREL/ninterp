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
    fn linear(&self, point: &[f64]) -> Result<f64, InterpolationError> {
        // x
        let x_l = find_nearest_index(&self.x, point[0]);
        let x_u = x_l + 1;
        let x_diff = (point[0] - self.x[x_l]) / (self.x[x_u] - self.x[x_l]);
        // y
        let y_l = find_nearest_index(&self.y, point[1]);
        let y_u = y_l + 1;
        let y_diff = (point[1] - self.y[y_l]) / (self.y[y_u] - self.y[y_l]);
        // z
        let z_l = find_nearest_index(&self.z, point[2]);
        let z_u = z_l + 1;
        let z_diff = (point[2] - self.z[z_l]) / (self.z[z_u] - self.z[z_l]);
        // interpolate in the x-direction
        let c00 = self.f_xyz[x_l][y_l][z_l] * (1.0 - x_diff) + self.f_xyz[x_u][y_l][z_l] * x_diff;
        let c01 = self.f_xyz[x_l][y_l][z_u] * (1.0 - x_diff) + self.f_xyz[x_u][y_l][z_u] * x_diff;
        let c10 = self.f_xyz[x_l][y_u][z_l] * (1.0 - x_diff) + self.f_xyz[x_u][y_u][z_l] * x_diff;
        let c11 = self.f_xyz[x_l][y_u][z_u] * (1.0 - x_diff) + self.f_xyz[x_u][y_u][z_u] * x_diff;
        // interpolate in the y-direction
        let c0 = c00 * (1.0 - y_diff) + c10 * y_diff;
        let c1 = c01 * (1.0 - y_diff) + c11 * y_diff;
        // interpolate in the z-direction
        Ok(c0 * (1.0 - z_diff) + c1 * z_diff)
    }
}

impl Nearest for Interp3D {
    fn nearest(&self, point: &[f64]) -> Result<f64, InterpolationError> {
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
    fn validate(&self) -> Result<(), ValidationError> {
        // Check that interpolation strategy is applicable
        if !matches!(self.strategy, Strategy::Linear | Strategy::Nearest) {
            return Err(ValidationError::StrategySelection(format!(
                "{:?}",
                self.strategy
            )));
        }

        // Check that extrapolation variant is applicable
        if matches!(self.extrapolate, Extrapolate::Enable) {
            return Err(ValidationError::ExtrapolationSelection(format!(
                "{:?}",
                self.extrapolate
            )));
        }

        // Check that each grid dimension has elements
        let x_grid_len = self.x.len();
        if x_grid_len == 0 {
            return Err(ValidationError::EmptyGrid("x".into()));
        }
        let y_grid_len = self.y.len();
        if y_grid_len == 0 {
            return Err(ValidationError::EmptyGrid("y".into()));
        }
        let z_grid_len = self.z.len();
        if z_grid_len == 0 {
            return Err(ValidationError::EmptyGrid("z".into()));
        }

        // Check that grid points are monotonically increasing
        if !self.x.windows(2).all(|w| w[0] <= w[1]) {
            return Err(ValidationError::Monotonicity("x".into()));
        }
        if !self.y.windows(2).all(|w| w[0] <= w[1]) {
            return Err(ValidationError::Monotonicity("y".into()));
        }
        if !self.z.windows(2).all(|w| w[0] <= w[1]) {
            return Err(ValidationError::Monotonicity("z".into()));
        }

        // Check that grid and values are compatible shapes
        if x_grid_len != self.f_xyz.len() {
            return Err(ValidationError::IncompatibleShapes("x".into()));
        }
        if !self
            .f_xyz
            .iter()
            .map(|y_vals| y_vals.len())
            .all(|y_val_len| y_val_len == y_grid_len)
        {
            return Err(ValidationError::IncompatibleShapes("y".into()));
        }
        if !self
            .f_xyz
            .iter()
            .flat_map(|y_vals| y_vals.iter().map(|z_vals| z_vals.len()))
            .all(|z_val_len| z_val_len == z_grid_len)
        {
            return Err(ValidationError::IncompatibleShapes("z".into()));
        }

        Ok(())
    }

    fn interpolate(&self, point: &[f64]) -> Result<f64, InterpolationError> {
        match self.strategy {
            Strategy::Linear => self.linear(point),
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
        ) // 3.2
    }

    #[test]
    fn test_extrapolate_inputs() {
        // Extrapolate::Extrapolate
        assert!(matches!(
            Interp3D {
                x: vec![0., 1.],
                y: vec![0., 1.],
                z: vec![0., 1.],
                f_xyz: vec![
                    vec![vec![0., 1.], vec![2., 3.]],
                    vec![vec![4., 5.], vec![6., 7.]],
                ],
                strategy: Strategy::Linear,
                extrapolate: Extrapolate::Enable,
            }
            .validate()
            .unwrap_err(),
            ValidationError::ExtrapolationSelection(_)
        ));
        // Extrapolate::Error
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
        assert!(matches!(
            interp.interpolate(&[-1., -1., -1.]).unwrap_err(),
            InterpolationError::ExtrapolationError(_)
        ));
        assert!(matches!(
            interp.interpolate(&[2., 2., 2.]).unwrap_err(),
            InterpolationError::ExtrapolationError(_)
        ));
    }

    #[test]
    fn test_extrapolate_clamp() {
        let interp = Interpolator::new_3d(
            vec![0., 1.],
            vec![0., 1.],
            vec![0., 1.],
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
