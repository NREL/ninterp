//! 2-dimensional interpolation

use super::*;

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Interp2D {
    pub(crate) x: Vec<f64>,
    pub(crate) y: Vec<f64>,
    pub(crate) f_xy: Vec<Vec<f64>>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub strategy: Strategy,
    #[cfg_attr(feature = "serde", serde(default))]
    pub extrapolate: Extrapolate,
}

impl Interp2D {
    /// Create and validate 2-D interpolator
    pub fn new(
        x: Vec<f64>,
        y: Vec<f64>,
        f_xy: Vec<Vec<f64>>,
        strategy: Strategy,
        extrapolate: Extrapolate,
    ) -> Result<Self, ValidationError> {
        let interp = Self {
            x,
            y,
            f_xy,
            strategy,
            extrapolate,
        };
        interp.validate()?;
        Ok(interp)
    }

    /// Function to set x variable from Interp2D
    /// # Arguments
    /// - `new_x`: updated `x` variable to replace the current `x` variable
    pub fn set_x(&mut self, new_x: Vec<f64>) -> Result<(), ValidationError> {
        self.x = new_x;
        self.validate()
    }

    /// Function to set y variable from Interp2D
    /// # Arguments
    /// - `new_y`: updated `y` variable to replace the current `y` variable
    pub fn set_y(&mut self, new_y: Vec<f64>) -> Result<(), ValidationError> {
        self.y = new_y;
        self.validate()
    }

    /// Function to set f_xy variable from Interp2D
    /// # Arguments
    /// - `new_f_xy`: updated `f_xy` variable to replace the current `f_xy` variable
    pub fn set_f_xy(&mut self, new_f_xy: Vec<Vec<f64>>) -> Result<(), ValidationError> {
        self.f_xy = new_f_xy;
        self.validate()
    }
}

impl Linear for Interp2D {
    fn linear(&self, point: &[f64]) -> Result<f64, InterpolationError> {
        let x_l = find_nearest_index(&self.x, point[0]);
        let x_u = x_l + 1;
        let x_diff = (point[0] - self.x[x_l]) / (self.x[x_u] - self.x[x_l]);

        let y_l = find_nearest_index(&self.y, point[1]);
        let y_u = y_l + 1;
        let y_diff = (point[1] - self.y[y_l]) / (self.y[y_u] - self.y[y_l]);

        // interpolate in the x-direction
        let c0 = self.f_xy[x_l][y_l] * (1.0 - x_diff) + self.f_xy[x_u][y_l] * x_diff;
        let c1 = self.f_xy[x_l][y_u] * (1.0 - x_diff) + self.f_xy[x_u][y_u] * x_diff;

        // interpolate in the y-direction
        Ok(c0 * (1.0 - y_diff) + c1 * y_diff)
    }
}

impl InterpMethods for Interp2D {
    fn validate(&self) -> Result<(), ValidationError> {
        // Check that interpolation strategy is applicable
        if !matches!(self.strategy, Strategy::Linear) {
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
        let interp = Interpolator::Interp2D(
            Interp2D::new(
                x.clone(),
                y.clone(),
                f_xy.clone(),
                Strategy::Linear,
                Extrapolate::Error,
            )
            .unwrap(),
        );
        assert_eq!(interp.interpolate(&[x[2], y[1]]).unwrap(), 7.);
        assert_eq!(interp.interpolate(&[x[2], y[1]]).unwrap(), 7.);
    }

    #[test]
    fn test_linear_offset() {
        let interp = Interpolator::Interp2D(
            Interp2D::new(
                vec![0., 1.],
                vec![0., 1.],
                vec![vec![0., 1.], vec![2., 3.]],
                Strategy::Linear,
                Extrapolate::Error,
            )
            .unwrap(),
        );
        let interp_res = interp.interpolate(&[0.25, 0.65]).unwrap();
        assert_eq!(interp_res, 1.1500000000000001) // 1.15
    }

    #[test]
    fn test_extrapolate_inputs() {
        // Extrapolate::Extrapolate
        assert!(matches!(
            Interp2D::new(
                vec![0., 1.],
                vec![0., 1.],
                vec![vec![0., 1.], vec![2., 3.]],
                Strategy::Linear,
                Extrapolate::Enable,
            )
            .unwrap_err(),
            ValidationError::ExtrapolationSelection(_)
        ));
        // Extrapolate::Error
        let interp = Interpolator::Interp2D(
            Interp2D::new(
                vec![0., 1.],
                vec![0., 1.],
                vec![vec![0., 1.], vec![2., 3.]],
                Strategy::Linear,
                Extrapolate::Error,
            )
            .unwrap(),
        );
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
        let interp = Interpolator::Interp2D(
            Interp2D::new(
                vec![0., 1.],
                vec![0., 1.],
                vec![vec![0., 1.], vec![2., 3.]],
                Strategy::Linear,
                Extrapolate::Clamp,
            )
            .unwrap(),
        );
        assert_eq!(interp.interpolate(&[-1., -1.]).unwrap(), 0.);
        assert_eq!(interp.interpolate(&[2., 2.]).unwrap(), 3.);
    }
}
