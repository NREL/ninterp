//! 2-dimensional interpolation

use super::*;

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub(crate) struct Interp2D {
    pub(crate) x: Vec<f64>,
    pub(crate) y: Vec<f64>,
    pub(crate) f_xy: Vec<Vec<f64>>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub(crate) strategy: Strategy,
    #[cfg_attr(feature = "serde", serde(default))]
    pub(crate) extrapolate: Extrapolate,
}

impl Linear for Interp2D {
    fn linear(&self, point: &[f64]) -> Result<f64, InterpolationError> {
        // Extrapolation is checked previously in Interpolator::interpolate,
        // meaning:
        // - point is within grid bounds, or
        // - point is clamped, or
        // - extrapolation is enabled
        let grid = [&self.x, &self.y];
        let lowers: Vec<usize> = (0..2)
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
        // interpolate in the x-direction
        let f0 = self.f_xy[x_l][y_l] * (1.0 - x_diff) + self.f_xy[x_u][y_l] * x_diff;
        let f1 = self.f_xy[x_l][y_u] * (1.0 - x_diff) + self.f_xy[x_u][y_u] * x_diff;
        // interpolate in the y-direction
        Ok(f0 * (1.0 - y_diff) + f1 * y_diff)
    }
}

impl Nearest for Interp2D {
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

        Ok(self.f_xy[i][j])
    }
}

impl InterpMethods for Interp2D {
    fn validate(&self) -> Result<(), ValidationError> {
        // Check applicablitity of strategy and extrapolate
        match (&self.strategy, &self.extrapolate) {
            // inapplicable strategies
            (Strategy::LeftNearest | Strategy::RightNearest, _) => Err(
                ValidationError::StrategySelection(format!("{:?}", self.strategy)),
            ),
            // inapplicable combinations of strategy + extrapolate
            (Strategy::Nearest, Extrapolate::Enable) => Err(
                ValidationError::ExtrapolationSelection(format!("{:?}", self.extrapolate)),
            ),
            _ => Ok(()),
        }?;

        let x_grid_len = self.x.len();
        let y_grid_len = self.y.len();

        // Check that each grid dimension has elements
        if x_grid_len == 0 {
            return Err(ValidationError::EmptyGrid("x".into()));
        }
        if y_grid_len == 0 {
            return Err(ValidationError::EmptyGrid("y".into()));
        }

        // If using Extrapolate::Enable,
        // check that each grid dimension has at least two elements
        if matches!(self.extrapolate, Extrapolate::Enable) && (x_grid_len < 2 || y_grid_len < 2) {
            return Err(ValidationError::Other(
                "at least 2 data points are required for extrapolation".into(),
            ));
        }

        // Check that grid points are monotonically increasing
        if !self.x.windows(2).all(|w| w[0] <= w[1]) {
            return Err(ValidationError::Monotonicity("x".into()));
        }
        if !self.y.windows(2).all(|w| w[0] <= w[1]) {
            return Err(ValidationError::Monotonicity("y".into()));
        }

        // Check that grid and values are compatible shapes
        if x_grid_len != self.f_xy.len() {
            return Err(ValidationError::IncompatibleShapes("x".into()));
        }
        if !self
            .f_xy
            .iter()
            .map(|y_vals| y_vals.len())
            .all(|y_val_len| y_val_len == y_grid_len)
        {
            return Err(ValidationError::IncompatibleShapes("y".into()));
        }

        Ok(())
    }

    fn interpolate(&self, point: &[f64]) -> Result<f64, InterpolationError> {
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
        let f_xy = vec![vec![0., 1., 2.], vec![3., 4., 5.], vec![6., 7., 8.]];
        let interp = Interpolator::new_2d(
            x.clone(),
            y.clone(),
            f_xy.clone(),
            Strategy::Linear,
            Extrapolate::Error,
        )
        .unwrap();
        // Check that interpolating at grid points just retrieves the value
        for (i, x_i) in x.iter().enumerate() {
            for (j, y_j) in y.iter().enumerate() {
                assert_eq!(interp.interpolate(&[*x_i, *y_j]).unwrap(), f_xy[i][j]);
            }
        }
        assert_eq!(interp.interpolate(&[x[2], y[1]]).unwrap(), f_xy[2][1]);
        assert_eq!(interp.interpolate(&[0.075, 0.25]).unwrap(), 3.);
    }

    #[test]
    fn test_linear_offset() {
        let interp = Interpolator::new_2d(
            vec![0., 1.],
            vec![0., 1.],
            vec![vec![0., 1.], vec![2., 3.]],
            Strategy::Linear,
            Extrapolate::Error,
        )
        .unwrap();
        assert_eq!(
            interp.interpolate(&[0.25, 0.65]).unwrap(),
            1.1500000000000001 // 1.15
        );
    }

    #[test]
    fn test_nearest() {
        let x = vec![0.05, 0.10, 0.15];
        let y = vec![0.10, 0.20, 0.30];
        let f_xy = vec![vec![0., 1., 2.], vec![3., 4., 5.], vec![6., 7., 8.]];
        let interp = Interpolator::new_2d(
            x.clone(),
            y.clone(),
            f_xy.clone(),
            Strategy::Nearest,
            Extrapolate::Error,
        )
        .unwrap();
        // Check that interpolating at grid points just retrieves the value
        for (i, x_i) in x.iter().enumerate() {
            for (j, y_j) in y.iter().enumerate() {
                assert_eq!(interp.interpolate(&[*x_i, *y_j]).unwrap(), f_xy[i][j]);
            }
        }
        assert_eq!(interp.interpolate(&[0.05, 0.12]).unwrap(), f_xy[0][0]);
        assert_eq!(
            // float imprecision
            interp.interpolate(&[0.07, 0.15 + 0.0001]).unwrap(),
            f_xy[0][1]
        );
        assert_eq!(interp.interpolate(&[0.08, 0.21]).unwrap(), f_xy[1][1]);
        assert_eq!(interp.interpolate(&[0.11, 0.26]).unwrap(), f_xy[1][2]);
        assert_eq!(interp.interpolate(&[0.13, 0.12]).unwrap(), f_xy[2][0]);
        assert_eq!(interp.interpolate(&[0.14, 0.29]).unwrap(), f_xy[2][2]);
    }

    #[test]
    fn test_extrapolate_inputs() {
        // Extrapolate::Extrapolate
        assert!(matches!(
            Interp2D {
                x: vec![0., 1.],
                y: vec![0., 1.],
                f_xy: vec![vec![0., 1.], vec![2., 3.]],
                strategy: Strategy::Nearest,
                extrapolate: Extrapolate::Enable,
            }
            .validate()
            .unwrap_err(),
            ValidationError::ExtrapolationSelection(_)
        ));
        // Extrapolate::Error
        let interp = Interpolator::new_2d(
            vec![0., 1.],
            vec![0., 1.],
            vec![vec![0., 1.], vec![2., 3.]],
            Strategy::Linear,
            Extrapolate::Error,
        )
        .unwrap();
        assert!(matches!(
            interp.interpolate(&[-1., -1.]).unwrap_err(),
            InterpolationError::ExtrapolationError(_)
        ));
        assert!(matches!(
            interp.interpolate(&[2., 2.]).unwrap_err(),
            InterpolationError::ExtrapolationError(_)
        ));
    }

    #[test]
    fn test_extrapolate_clamp() {
        let interp = Interpolator::new_2d(
            vec![0., 1.],
            vec![0., 1.],
            vec![vec![0., 1.], vec![2., 3.]],
            Strategy::Linear,
            Extrapolate::Clamp,
        )
        .unwrap();
        assert_eq!(interp.interpolate(&[-1., -1.]).unwrap(), 0.);
        assert_eq!(interp.interpolate(&[2., 2.]).unwrap(), 3.);
    }
}
