//! 1-dimensional interpolation

use super::*;

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Interp1D {
    pub(crate) x: Vec<f64>,
    pub(crate) f_x: Vec<f64>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub strategy: Strategy,
    #[cfg_attr(feature = "serde", serde(default))]
    pub extrapolate: Extrapolate,
}

impl Interp1D {
    /// Create and validate 1-D interpolator
    pub fn new(
        x: Vec<f64>,
        f_x: Vec<f64>,
        strategy: Strategy,
        extrapolate: Extrapolate,
    ) -> Result<Self, ValidationError> {
        let interp = Self {
            x,
            f_x,
            strategy,
            extrapolate,
        };
        interp.validate()?;
        Ok(interp)
    }

    /// Function to set x variable from Interp1D
    /// # Arguments
    /// - `new_x`: updated `x` variable to replace the current `x` variable
    pub fn set_x(&mut self, new_x: Vec<f64>) -> Result<(), ValidationError> {
        self.x = new_x;
        self.validate()
    }

    /// Function to set f_x variable from Interp1D
    /// # Arguments
    /// - `new_f_x`: updated `f_x` variable to replace the current `f_x` variable
    pub fn set_f_x(&mut self, new_f_x: Vec<f64>) -> Result<(), ValidationError> {
        self.f_x = new_f_x;
        self.validate()
    }
}

impl Linear for Interp1D {
    fn linear(&self, point: &[f64]) -> Result<f64, InterpolationError> {
        if let Some(i) = self.x.iter().position(|&x_val| x_val == point[0]) {
            return Ok(self.f_x[i]);
        }
        // Extrapolate, if applicable
        if matches!(self.extrapolate, Extrapolate::Enable) {
            if point[0] < self.x[0] {
                let slope = (self.f_x[1] - self.f_x[0]) / (self.x[1] - self.x[0]);
                return Ok(slope * (point[0] - self.x[0]) + self.f_x[0]);
            } else if &point[0] > self.x.last().unwrap() {
                let slope = (self.f_x.last().unwrap() - self.f_x[self.f_x.len() - 2])
                    / (self.x.last().unwrap() - self.x[self.x.len() - 2]);
                return Ok(slope * (point[0] - self.x.last().unwrap()) + self.f_x.last().unwrap());
            }
        }
        let lower_index = find_nearest_index(&self.x, point[0]);
        let diff =
            (point[0] - self.x[lower_index]) / (self.x[lower_index + 1] - self.x[lower_index]);
        Ok(self.f_x[lower_index] * (1.0 - diff) + self.f_x[lower_index + 1] * diff)
    }
}

impl LeftNearest for Interp1D {
    fn left_nearest(&self, point: &[f64]) -> Result<f64, InterpolationError> {
        if let Some(i) = self.x.iter().position(|&x_val| x_val == point[0]) {
            return Ok(self.f_x[i]);
        }
        let lower_index = find_nearest_index(&self.x, point[0]);
        Ok(self.f_x[lower_index])
    }
}

impl RightNearest for Interp1D {
    fn right_nearest(&self, point: &[f64]) -> Result<f64, InterpolationError> {
        if let Some(i) = self.x.iter().position(|&x_val| x_val == point[0]) {
            return Ok(self.f_x[i]);
        }
        let lower_index = find_nearest_index(&self.x, point[0]);
        Ok(self.f_x[lower_index + 1])
    }
}

impl Nearest for Interp1D {
    fn nearest(&self, point: &[f64]) -> Result<f64, InterpolationError> {
        if let Some(i) = self.x.iter().position(|&x_val| x_val == point[0]) {
            return Ok(self.f_x[i]);
        }
        let lower_index = find_nearest_index(&self.x, point[0]);
        let diff =
            (point[0] - self.x[lower_index]) / (self.x[lower_index + 1] - self.x[lower_index]);
        Ok(if diff < 0.5 {
            self.f_x[lower_index]
        } else {
            self.f_x[lower_index + 1]
        })
    }
}

impl InterpMethods for Interp1D {
    fn validate(&self) -> Result<(), ValidationError> {
        let x_grid_len = self.x.len();

        // Check that extrapolation variant is applicable
        if matches!(self.extrapolate, Extrapolate::Enable) {
            if !matches!(self.strategy, Strategy::Linear) {
                return Err(ValidationError::ExtrapolationSelection(format!(
                    "{:?}",
                    self.extrapolate
                )));
            }
            if x_grid_len < 2 {
                return Err(ValidationError::Other(
                    "At least 2 data points are required for extrapolation".into(),
                ));
            }
        }

        // Check that each grid dimension has elements
        if x_grid_len == 0 {
            return Err(ValidationError::EmptyGrid("x".into()));
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
        let interp = Interpolator::Interp1D(
            Interp1D::new(
                vec![0., 1., 2., 3., 4.],
                vec![0.2, 0.4, 0.6, 0.8, 1.0],
                Strategy::Linear,
                Extrapolate::Error,
            )
            .unwrap(),
        );
        assert!(matches!(
            interp.interpolate(&[]).unwrap_err(),
            InterpolationError::InvalidPoint(_)
        ));
        assert_eq!(interp.interpolate(&[1.0]).unwrap(), 0.4);
    }

    #[test]
    fn test_linear() {
        let interp = Interpolator::Interp1D(
            Interp1D::new(
                vec![0., 1., 2., 3., 4.],
                vec![0.2, 0.4, 0.6, 0.8, 1.0],
                Strategy::Linear,
                Extrapolate::Error,
            )
            .unwrap(),
        );
        assert_eq!(interp.interpolate(&[3.00]).unwrap(), 0.8);
        assert_eq!(interp.interpolate(&[3.75]).unwrap(), 0.95);
        assert_eq!(interp.interpolate(&[4.00]).unwrap(), 1.0);
    }

    #[test]
    fn test_left_nearest() {
        let interp = Interpolator::Interp1D(
            Interp1D::new(
                vec![0., 1., 2., 3., 4.],
                vec![0.2, 0.4, 0.6, 0.8, 1.0],
                Strategy::LeftNearest,
                Extrapolate::Error,
            )
            .unwrap(),
        );
        assert_eq!(interp.interpolate(&[3.00]).unwrap(), 0.8);
        assert_eq!(interp.interpolate(&[3.75]).unwrap(), 0.8);
        assert_eq!(interp.interpolate(&[4.00]).unwrap(), 1.0);
    }

    #[test]
    fn test_right_nearest() {
        let interp = Interpolator::Interp1D(
            Interp1D::new(
                vec![0., 1., 2., 3., 4.],
                vec![0.2, 0.4, 0.6, 0.8, 1.0],
                Strategy::RightNearest,
                Extrapolate::Error,
            )
            .unwrap(),
        );
        assert_eq!(interp.interpolate(&[3.00]).unwrap(), 0.8);
        assert_eq!(interp.interpolate(&[3.25]).unwrap(), 1.0);
        assert_eq!(interp.interpolate(&[4.00]).unwrap(), 1.0);
    }

    #[test]
    fn test_nearest() {
        let interp = Interpolator::Interp1D(
            Interp1D::new(
                vec![0., 1., 2., 3., 4.],
                vec![0.2, 0.4, 0.6, 0.8, 1.0],
                Strategy::Nearest,
                Extrapolate::Error,
            )
            .unwrap(),
        );
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
                Strategy::Nearest,
                Extrapolate::Enable,
            )
            .unwrap_err(),
            ValidationError::ExtrapolationSelection(_)
        ));

        // Extrapolate::Error
        let interp = Interpolator::Interp1D(
            Interp1D::new(
                vec![0., 1., 2., 3., 4.],
                vec![0.2, 0.4, 0.6, 0.8, 1.0],
                Strategy::Linear,
                Extrapolate::Error,
            )
            .unwrap(),
        );
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
        let interp = Interpolator::Interp1D(
            Interp1D::new(
                vec![0., 1., 2., 3., 4.],
                vec![0.2, 0.4, 0.6, 0.8, 1.0],
                Strategy::Linear,
                Extrapolate::Clamp,
            )
            .unwrap(),
        );
        assert_eq!(interp.interpolate(&[-1.]).unwrap(), 0.2);
        assert_eq!(interp.interpolate(&[5.]).unwrap(), 1.0);
    }

    #[test]
    fn test_extrapolate() {
        let interp = Interpolator::Interp1D(
            Interp1D::new(
                vec![0., 1., 2., 3., 4.],
                vec![0.2, 0.4, 0.6, 0.8, 1.0],
                Strategy::Linear,
                Extrapolate::Enable,
            )
            .unwrap(),
        );
        assert_eq!(interp.interpolate(&[-1.]).unwrap(), 0.0);
        assert_eq!(interp.interpolate(&[5.]).unwrap(), 1.2);
    }
}
