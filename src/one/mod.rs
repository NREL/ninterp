//! 1-dimensional interpolation

use super::*;

mod strategies;

const N: usize = 1;

#[non_exhaustive]
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Interp1D {
    pub x: Vec<f64>,
    pub f_x: Vec<f64>,
    pub strategy: Box<dyn Interp1DStrategy>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub extrapolate: Extrapolate,
}

impl Interp1D {
    pub fn new(
        x: Vec<f64>,
        f_x: Vec<f64>,
        strategy: impl Interp1DStrategy + 'static,
        extrapolate: Extrapolate,
    ) -> Result<Self, ValidateError> {
        let interpolator = Self {
            x,
            f_x,
            strategy: Box::new(strategy),
            extrapolate,
        };
        interpolator.validate()?;
        Ok(interpolator)
    }

    pub fn set_strategy(
        &mut self,
        strategy: impl Interp1DStrategy + 'static,
    ) -> Result<(), ValidateError> {
        self.strategy = Box::new(strategy);
        self.check_extrapolate(self.extrapolate)
    }

    fn check_extrapolate(&self, extrapolate: Extrapolate) -> Result<(), ValidateError> {
        // Check applicability of strategy and extrapolate setting
        if matches!(extrapolate, Extrapolate::Enable) && !self.strategy.allow_extrapolate() {
            return Err(ValidateError::ExtrapolateSelection(self.extrapolate));
        }
        // If using Extrapolate::Enable,
        // check that each grid dimension has at least two elements
        if matches!(self.extrapolate, Extrapolate::Enable) && self.x.len() < 2 {
            return Err(ValidateError::Other(
                "at least 2 data points are required for extrapolation".into(),
            ));
        }
        Ok(())
    }
}

impl Interpolator for Interp1D {
    /// Returns `1`
    fn ndim(&self) -> usize {
        N
    }

    fn validate(&self) -> Result<(), ValidateError> {
        self.check_extrapolate(self.extrapolate)?;

        let x_grid_len = self.x.len();

        // Check that each grid dimension has elements
        if x_grid_len == 0 {
            return Err(ValidateError::EmptyGrid("x".into()));
        }

        // Check that grid points are monotonically increasing
        if !self.x.windows(2).all(|w| w[0] <= w[1]) {
            return Err(ValidateError::Monotonicity("x".into()));
        }

        // Check that grid and values are compatible shapes
        if x_grid_len != self.f_x.len() {
            return Err(ValidateError::IncompatibleShapes("x".into()));
        }

        Ok(())
    }

    fn interpolate(&self, point: &[f64]) -> Result<f64, InterpolateError> {
        let point: &[f64; N] = point
            .try_into()
            .map_err(|_| InterpolateError::PointLength(N))?;
        if !(self.x.first().unwrap()..=self.x.last().unwrap()).contains(&&point[0]) {
            match self.extrapolate {
                Extrapolate::Enable => {}
                Extrapolate::Fill(value) => return Ok(value),
                Extrapolate::Clamp => {
                    let clamped_point =
                        &[point[0].clamp(*self.x.first().unwrap(), *self.x.last().unwrap())];
                    return self.strategy.interpolate(self, clamped_point);
                }
                Extrapolate::Error => {
                    return Err(InterpolateError::ExtrapolateError(format!(
                        "\n    point[0] = {:?} is out of bounds for x-grid = {:?}",
                        point[0], self.x
                    )))
                }
            }
        };
        self.strategy.interpolate(self, point)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_args() {
        let interp = Interp1D::new(
            vec![0., 1., 2., 3., 4.],
            vec![0.2, 0.4, 0.6, 0.8, 1.0],
            Linear,
            Extrapolate::Error,
        )
        .unwrap();
        assert!(matches!(
            interp.interpolate(&[]).unwrap_err(),
            InterpolateError::PointLength(_)
        ));
        assert_eq!(interp.interpolate(&[1.0]).unwrap(), 0.4);
    }

    #[test]
    fn test_linear() {
        let x = vec![0., 1., 2., 3., 4.];
        let f_x = vec![0.2, 0.4, 0.6, 0.8, 1.0];
        let interp =
            Interp1D::new(x.clone(), f_x.clone(), Linear, Extrapolate::Error).unwrap();
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
        let interp =
            Interp1D::new(x.clone(), f_x.clone(), LeftNearest, Extrapolate::Error).unwrap();
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
        let interp =
            Interp1D::new(x.clone(), f_x.clone(), RightNearest, Extrapolate::Error).unwrap();
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
        let interp =
            Interp1D::new(x.clone(), f_x.clone(), Nearest, Extrapolate::Error).unwrap();
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
            Interp1D::new(
                vec![0., 1., 2., 3., 4.],
                vec![0.2, 0.4, 0.6, 0.8, 1.0],
                Nearest,
                Extrapolate::Enable,
            )
            .unwrap_err(),
            ValidateError::ExtrapolateSelection(_)
        ));

        // Extrapolate::Error
        let interp = Interp1D::new(
            vec![0., 1., 2., 3., 4.],
            vec![0.2, 0.4, 0.6, 0.8, 1.0],
            Linear,
            Extrapolate::Error,
        )
        .unwrap();
        // Fail to extrapolate below lowest grid value
        assert!(matches!(
            interp.interpolate(&[-1.]).unwrap_err(),
            InterpolateError::ExtrapolateError(_)
        ));
        // Fail to extrapolate above highest grid value
        assert!(matches!(
            interp.interpolate(&[5.]).unwrap_err(),
            InterpolateError::ExtrapolateError(_)
        ));
    }

    #[test]
    fn test_extrapolate_fill_value() {
        let interp = Interp1D::new(
            vec![0., 1., 2., 3., 4.],
            vec![0.2, 0.4, 0.6, 0.8, 1.0],
            Linear,
            Extrapolate::Fill(f64::NAN),
        )
        .unwrap();
        assert_eq!(interp.interpolate(&[1.5]).unwrap(), 0.5);
        assert_eq!(interp.interpolate(&[2.]).unwrap(), 0.6);
        assert!(interp.interpolate(&[-1.]).unwrap().is_nan());
        assert!(interp.interpolate(&[5.]).unwrap().is_nan());
    }

    #[test]
    fn test_extrapolate_clamp() {
        let interp = Interp1D::new(
            vec![0., 1., 2., 3., 4.],
            vec![0.2, 0.4, 0.6, 0.8, 1.0],
            Linear,
            Extrapolate::Clamp,
        )
        .unwrap();
        assert_eq!(interp.interpolate(&[-1.]).unwrap(), 0.2);
        assert_eq!(interp.interpolate(&[5.]).unwrap(), 1.0);
    }

    #[test]
    fn test_extrapolate() {
        let interp = Interp1D::new(
            vec![0., 1., 2., 3., 4.],
            vec![0.2, 0.4, 0.6, 0.8, 1.0],
            Linear,
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
