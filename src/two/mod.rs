//! 2-dimensional interpolation

use super::*;

mod strategies;

const N: usize = 2;

#[non_exhaustive]
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Data2D {
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub f_xy: Vec<Vec<f64>>,
}

/// 2-D interpolator
#[non_exhaustive]
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Interp2D<S: Strategy2D> {
    pub data: Data2D,
    pub strategy: S,
    #[cfg_attr(feature = "serde", serde(default))]
    pub extrapolate: Extrapolate,
}

impl<S: Strategy2D> Interp2D<S> {
    pub fn new(
        x: Vec<f64>,
        y: Vec<f64>,
        f_xy: Vec<Vec<f64>>,
        strategy: S,
        extrapolate: Extrapolate,
    ) -> Result<Self, ValidateError> {
        let interpolator = Self {
            data: Data2D { x, y, f_xy },
            strategy,
            extrapolate,
        };
        interpolator.validate()?;
        Ok(interpolator)
    }

    pub fn set_strategy(&mut self, strategy: S) -> Result<(), ValidateError> {
        self.strategy = strategy;
        self.check_extrapolate(self.extrapolate)
    }

    pub fn set_extrapolate(&mut self, extrapolate: Extrapolate) -> Result<(), ValidateError> {
        self.check_extrapolate(extrapolate)?;
        self.extrapolate = extrapolate;
        Ok(())
    }

    fn check_extrapolate(&self, extrapolate: Extrapolate) -> Result<(), ValidateError> {
        // Check applicability of strategy and extrapolate setting
        if matches!(extrapolate, Extrapolate::Enable) && !self.strategy.allow_extrapolate() {
            return Err(ValidateError::ExtrapolateSelection(self.extrapolate));
        }
        // If using Extrapolate::Enable,
        // check that each grid dimension has at least two elements
        if matches!(self.extrapolate, Extrapolate::Enable)
            && (self.data.x.len() < 2 || self.data.y.len() < 2)
        {
            return Err(ValidateError::Other(
                "at least 2 data points are required for extrapolation".into(),
            ));
        }
        Ok(())
    }
}

impl<S: Strategy2D> Interpolator for Interp2D<S> {
    /// Returns `2`
    fn ndim(&self) -> usize {
        N
    }

    fn validate(&self) -> Result<(), ValidateError> {
        self.check_extrapolate(self.extrapolate)?;

        let x_grid_len = self.data.x.len();
        let y_grid_len = self.data.y.len();

        // Check that each grid dimension has elements
        if x_grid_len == 0 {
            return Err(ValidateError::EmptyGrid("x".into()));
        }
        if y_grid_len == 0 {
            return Err(ValidateError::EmptyGrid("y".into()));
        }

        // Check that grid points are monotonically increasing
        if !self.data.x.windows(2).all(|w| w[0] <= w[1]) {
            return Err(ValidateError::Monotonicity("x".into()));
        }
        if !self.data.y.windows(2).all(|w| w[0] <= w[1]) {
            return Err(ValidateError::Monotonicity("y".into()));
        }

        // Check that grid and values are compatible shapes
        if x_grid_len != self.data.f_xy.len() {
            return Err(ValidateError::IncompatibleShapes("x".into()));
        }
        if !self
            .data
            .f_xy
            .iter()
            .map(Vec::len)
            .all(|y_val_len| y_val_len == y_grid_len)
        {
            return Err(ValidateError::IncompatibleShapes("y".into()));
        }

        Ok(())
    }

    fn interpolate(&self, point: &[f64]) -> Result<f64, InterpolateError> {
        let point: &[f64; N] = point
            .try_into()
            .map_err(|_| InterpolateError::PointLength(N))?;
        let grid = [&self.data.x, &self.data.y];
        let grid_names = ["x", "y"];
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear() {
        let x = vec![0.05, 0.10, 0.15];
        let y = vec![0.10, 0.20, 0.30];
        let f_xy = vec![vec![0., 1., 2.], vec![3., 4., 5.], vec![6., 7., 8.]];
        let interp = Interp2D::new(
            x.clone(),
            y.clone(),
            f_xy.clone(),
            Linear,
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
        let interp = Interp2D::new(
            vec![0., 1.],
            vec![0., 1.],
            vec![vec![0., 1.], vec![2., 3.]],
            Linear,
            Extrapolate::Error,
        )
        .unwrap();
        assert_eq!(
            interp.interpolate(&[0.25, 0.65]).unwrap(),
            1.1500000000000001 // 1.15
        );
    }

    #[test]
    fn test_linear_extrapolation() {
        let interp = Interp2D::new(
            vec![0.05, 0.10, 0.15],
            vec![0.10, 0.20, 0.30],
            vec![vec![0., 1., 2.], vec![3., 4., 5.], vec![6., 7., 8.]],
            Linear,
            Extrapolate::Enable,
        )
        .unwrap();
        // RHS are coplanar neighboring data planes according to:
        // https://www.ambrbit.com/TrigoCalc/Plan3D/PointsCoplanar.htm
        // below x, below y
        assert_eq!(interp.interpolate(&[0.0, 0.0]).unwrap(), -4.);
        assert_eq!(
            interp.interpolate(&[0.03, 0.04]).unwrap(),
            -1.8000000000000003
        );
        // below x, above y
        assert_eq!(
            interp.interpolate(&[0.0, 0.32]).unwrap(),
            -0.7999999999999998
        );
        assert_eq!(interp.interpolate(&[0.03, 0.36]).unwrap(), 1.4);
        // above x, below y
        assert_eq!(interp.interpolate(&[0.17, 0.0]).unwrap(), 6.200000000000001);
        assert_eq!(
            interp.interpolate(&[0.19, 0.04]).unwrap(),
            7.800000000000002
        );
        // above x, above y
        assert_eq!(interp.interpolate(&[0.17, 0.32]).unwrap(), 9.4);
        assert_eq!(interp.interpolate(&[0.19, 0.36]).unwrap(), 11.);
    }

    #[test]
    fn test_nearest() {
        let x = vec![0.05, 0.10, 0.15];
        let y = vec![0.10, 0.20, 0.30];
        let f_xy = vec![vec![0., 1., 2.], vec![3., 4., 5.], vec![6., 7., 8.]];
        let interp = Interp2D::new(
            x.clone(),
            y.clone(),
            f_xy.clone(),
            Nearest,
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
            Interp2D::new(
                vec![0.1, 1.1],
                vec![0.2, 1.2],
                vec![vec![0., 1.], vec![2., 3.]],
                Nearest,
                Extrapolate::Enable,
            )
            .unwrap_err(),
            ValidateError::ExtrapolateSelection(_)
        ));
        // Extrapolate::Error
        let interp = Interp2D::new(
            vec![0.1, 1.1],
            vec![0.2, 1.2],
            vec![vec![0., 1.], vec![2., 3.]],
            Linear,
            Extrapolate::Error,
        )
        .unwrap();
        assert!(matches!(
            interp.interpolate(&[-1., -1.]).unwrap_err(),
            InterpolateError::ExtrapolateError(_)
        ));
        assert!(matches!(
            interp.interpolate(&[2., 2.]).unwrap_err(),
            InterpolateError::ExtrapolateError(_)
        ));
    }

    #[test]
    fn test_extrapolate_fill_value() {
        let interp = Interp2D::new(
            vec![0.1, 1.1],
            vec![0.2, 1.2],
            vec![vec![0., 1.], vec![2., 3.]],
            Linear,
            Extrapolate::Fill(f64::NAN),
        )
        .unwrap();
        assert_eq!(interp.interpolate(&[0.5, 0.5]).unwrap(), 1.1);
        assert_eq!(interp.interpolate(&[0.1, 1.2]).unwrap(), 1.);
        assert!(interp.interpolate(&[0., 0.]).unwrap().is_nan());
        assert!(interp.interpolate(&[0., 2.]).unwrap().is_nan());
        assert!(interp.interpolate(&[2., 0.]).unwrap().is_nan());
        assert!(interp.interpolate(&[2., 2.]).unwrap().is_nan());
    }

    #[test]
    fn test_dyn_strategy() {
        let mut interp = Interp2D::new(
            vec![0., 1.],
            vec![0., 1.],
            vec![vec![0., 1.], vec![2., 3.]],
            Box::new(Linear) as Box<dyn Strategy2D>,
            Extrapolate::Error,
        )
        .unwrap();
        assert_eq!(interp.interpolate(&[0.2, 0.]).unwrap(), 0.4);
        interp.set_strategy(Box::new(Nearest)).unwrap();
        assert_eq!(interp.interpolate(&[0.2, 0.]).unwrap(), 0.);
    }

    #[test]
    fn test_extrapolate_clamp() {
        let interp = Interp2D::new(
            vec![0.1, 1.1],
            vec![0.2, 1.2],
            vec![vec![0., 1.], vec![2., 3.]],
            Linear,
            Extrapolate::Clamp,
        )
        .unwrap();
        assert_eq!(interp.interpolate(&[-1., -1.]).unwrap(), 0.);
        assert_eq!(interp.interpolate(&[2., 2.]).unwrap(), 3.);
    }
}
