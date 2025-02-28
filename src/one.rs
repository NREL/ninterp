//! 1-dimensional interpolation

use super::*;

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub(crate) struct Interp1D {
    pub(crate) x: Vec<f64>,
    pub(crate) f_x: Vec<f64>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub(crate) strategy: Strategy,
    #[cfg_attr(feature = "serde", serde(default))]
    pub(crate) extrapolate: Extrapolate,
}

impl Linear for Interp1D {
    fn linear(&self, point: &[f64]) -> Result<f64, InterpolationError> {
        if let Some(i) = self.x.iter().position(|&x_val| x_val == point[0]) {
            return Ok(self.f_x[i]);
        }
        // Extrapolation is checked previously in `Interpolator::interpolate`,
        // meaning:
        // - point is within grid bounds, or
        // - point is clamped, or
        // - extrapolation is enabled
        let x_l = if &point[0] < self.x.first().unwrap() {
            0
        } else if &point[0] > self.x.last().unwrap() {
            self.x.len() - 2
        } else {
            find_nearest_index(&self.x, point[0])
        };
        let x_u = x_l + 1;
        let x_diff = (point[0] - self.x[x_l]) / (self.x[x_u] - self.x[x_l]);
        Ok(self.f_x[x_l] * (1.0 - x_diff) + self.f_x[x_u] * x_diff)
    }
}

impl LeftNearest for Interp1D {
    fn left_nearest(&self, point: &[f64]) -> Result<f64, InterpolationError> {
        if let Some(i) = self.x.iter().position(|&x_val| x_val == point[0]) {
            return Ok(self.f_x[i]);
        }
        let x_l = find_nearest_index(&self.x, point[0]);
        Ok(self.f_x[x_l])
    }
}

impl RightNearest for Interp1D {
    fn right_nearest(&self, point: &[f64]) -> Result<f64, InterpolationError> {
        if let Some(i) = self.x.iter().position(|&x_val| x_val == point[0]) {
            return Ok(self.f_x[i]);
        }
        let x_u = find_nearest_index(&self.x, point[0]) + 1;
        Ok(self.f_x[x_u])
    }
}

impl Nearest for Interp1D {
    fn nearest(&self, point: &[f64]) -> Result<f64, InterpolationError> {
        if let Some(i) = self.x.iter().position(|&x_val| x_val == point[0]) {
            return Ok(self.f_x[i]);
        }
        let x_l = find_nearest_index(&self.x, point[0]);
        let x_u = x_l + 1;
        let x_diff = (point[0] - self.x[x_l]) / (self.x[x_u] - self.x[x_l]);
        let i = if x_diff < 0.5 { x_l } else { x_u };
        Ok(self.f_x[i])
    }
}

impl InterpMethods for Interp1D {
    fn validate(&self) -> Result<(), ValidationError> {
        // Check applicablitity of strategy and extrapolate
        match (&self.strategy, &self.extrapolate) {
            // inapplicable combinations of strategy + extrapolate
            (
                Strategy::LeftNearest | Strategy::RightNearest | Strategy::Nearest,
                Extrapolate::Enable,
            ) => Err(ValidationError::StrategySelection(format!(
                "{:?}",
                self.strategy
            ))),
            _ => Ok(()),
        }?;

        let x_grid_len = self.x.len();

        // Check that each grid dimension has elements
        if x_grid_len == 0 {
            return Err(ValidationError::EmptyGrid("x".into()));
        }

        // If using Extrapolate::Enable,
        // check that each grid dimension has at least two elements
        if matches!(self.extrapolate, Extrapolate::Enable) && x_grid_len < 2 {
            return Err(ValidationError::Other(
                "at least 2 data points are required for extrapolation".into(),
            ));
        }

        // Check that grid points are monotonically increasing
        if !self.x.windows(2).all(|w| w[0] <= w[1]) {
            return Err(ValidationError::Monotonicity("x".into()));
        }

        // Check that grid and values are compatible shapes
        if x_grid_len != self.f_x.len() {
            return Err(ValidationError::IncompatibleShapes("x".into()));
        }

        Ok(())
    }

    fn interpolate(&self, point: &[f64]) -> Result<f64, InterpolationError> {
        match self.strategy {
            Strategy::Linear => self.linear(point),
            Strategy::LeftNearest => self.left_nearest(point),
            Strategy::RightNearest => self.right_nearest(point),
            Strategy::Nearest => self.nearest(point),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_args() {
        let interp = Interpolator::new_1d(
            vec![0., 1., 2., 3., 4.],
            vec![0.2, 0.4, 0.6, 0.8, 1.0],
            Strategy::Linear,
            Extrapolate::Error,
        )
        .unwrap();
        assert!(matches!(
            interp.interpolate(&[]).unwrap_err(),
            InterpolationError::InvalidPoint(_)
        ));
        assert_eq!(interp.interpolate(&[1.0]).unwrap(), 0.4);
    }

    #[test]
    fn test_linear() {
        let x = vec![0., 1., 2., 3., 4.];
        let f_x = vec![0.2, 0.4, 0.6, 0.8, 1.0];
        let interp =
            Interpolator::new_1d(x.clone(), f_x.clone(), Strategy::Linear, Extrapolate::Error)
                .unwrap();
        // Check that interpolating at grid points just retrieves the value
        for (i, x_i) in x.iter().enumerate() {
            assert_eq!(interp.interpolate(&[*x_i]).unwrap(), f_x[i]);
        }
        assert_eq!(interp.interpolate(&[3.00]).unwrap(), 0.8);
        assert_eq!(interp.interpolate(&[3.75]).unwrap(), 0.95);
        assert_eq!(interp.interpolate(&[4.00]).unwrap(), 1.0);
    }

    #[test]
    fn test_left_nearest() {
        let x = vec![0., 1., 2., 3., 4.];
        let f_x = vec![0.2, 0.4, 0.6, 0.8, 1.0];
        let interp = Interpolator::new_1d(
            x.clone(),
            f_x.clone(),
            Strategy::LeftNearest,
            Extrapolate::Error,
        )
        .unwrap();
        // Check that interpolating at grid points just retrieves the value
        for (i, x_i) in x.iter().enumerate() {
            assert_eq!(interp.interpolate(&[*x_i]).unwrap(), f_x[i]);
        }
        assert_eq!(interp.interpolate(&[3.00]).unwrap(), 0.8);
        assert_eq!(interp.interpolate(&[3.75]).unwrap(), 0.8);
        assert_eq!(interp.interpolate(&[4.00]).unwrap(), 1.0);
    }

    #[test]
    fn test_right_nearest() {
        let x = vec![0., 1., 2., 3., 4.];
        let f_x = vec![0.2, 0.4, 0.6, 0.8, 1.0];
        let interp = Interpolator::new_1d(
            x.clone(),
            f_x.clone(),
            Strategy::RightNearest,
            Extrapolate::Error,
        )
        .unwrap();
        // Check that interpolating at grid points just retrieves the value
        for (i, x_i) in x.iter().enumerate() {
            assert_eq!(interp.interpolate(&[*x_i]).unwrap(), f_x[i]);
        }
        assert_eq!(interp.interpolate(&[3.00]).unwrap(), 0.8);
        assert_eq!(interp.interpolate(&[3.25]).unwrap(), 1.0);
        assert_eq!(interp.interpolate(&[4.00]).unwrap(), 1.0);
    }

    #[test]
    fn test_nearest() {
        let x = vec![0., 1., 2., 3., 4.];
        let f_x = vec![0.2, 0.4, 0.6, 0.8, 1.0];
        let interp = Interpolator::new_1d(
            x.clone(),
            f_x.clone(),
            Strategy::Nearest,
            Extrapolate::Error,
        )
        .unwrap();
        // Check that interpolating at grid points just retrieves the value
        for (i, x_i) in x.iter().enumerate() {
            assert_eq!(interp.interpolate(&[*x_i]).unwrap(), f_x[i]);
        }
        assert_eq!(interp.interpolate(&[3.00]).unwrap(), 0.8);
        assert_eq!(interp.interpolate(&[3.25]).unwrap(), 0.8);
        assert_eq!(interp.interpolate(&[3.50]).unwrap(), 1.0);
        assert_eq!(interp.interpolate(&[3.75]).unwrap(), 1.0);
        assert_eq!(interp.interpolate(&[4.00]).unwrap(), 1.0);
    }

    #[test]
    fn test_extrapolate_inputs() {
        // Incorrect extrapolation selection
        assert!(matches!(
            Interp1D {
                x: vec![0., 1., 2., 3., 4.],
                f_x: vec![0.2, 0.4, 0.6, 0.8, 1.0],
                strategy: Strategy::Nearest,
                extrapolate: Extrapolate::Enable,
            }
            .validate()
            .unwrap_err(),
            ValidationError::ExtrapolationSelection(_)
        ));

        // Extrapolate::Error
        let interp = Interpolator::new_1d(
            vec![0., 1., 2., 3., 4.],
            vec![0.2, 0.4, 0.6, 0.8, 1.0],
            Strategy::Linear,
            Extrapolate::Error,
        )
        .unwrap();
        // Fail to extrapolate below lowest grid value
        assert!(matches!(
            interp.interpolate(&[-1.]).unwrap_err(),
            InterpolationError::ExtrapolationError(_)
        ));
        // Fail to extrapolate above highest grid value
        assert!(matches!(
            interp.interpolate(&[5.]).unwrap_err(),
            InterpolationError::ExtrapolationError(_)
        ));
    }

    #[test]
    fn test_extrapolate_clamp() {
        let interp = Interpolator::new_1d(
            vec![0., 1., 2., 3., 4.],
            vec![0.2, 0.4, 0.6, 0.8, 1.0],
            Strategy::Linear,
            Extrapolate::Clamp,
        )
        .unwrap();
        assert_eq!(interp.interpolate(&[-1.]).unwrap(), 0.2);
        assert_eq!(interp.interpolate(&[5.]).unwrap(), 1.0);
    }

    #[test]
    fn test_extrapolate() {
        let interp = Interpolator::new_1d(
            vec![0., 1., 2., 3., 4.],
            vec![0.2, 0.4, 0.6, 0.8, 1.0],
            Strategy::Linear,
            Extrapolate::Enable,
        )
        .unwrap();
        assert_eq!(interp.interpolate(&[-1.]).unwrap(), 0.0);
        assert_eq!(
            interp.interpolate(&[-0.75]).unwrap(),
            0.04999999999999999 // 0.05
        );
        assert_eq!(interp.interpolate(&[5.]).unwrap(), 1.2);
    }
}
