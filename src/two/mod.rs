//! 2-dimensional interpolation

use super::*;

mod strategies;

const N: usize = 2;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct InterpData2D<T> {
    pub grid: [Array1<T>; N],
    pub values: Array<T, Dim<[Ix; N]>>,
}
validate_impl!(InterpData2D<T>);
impl<T: Num + PartialOrd + Copy + Debug> InterpData2D<T> {
    /// Returns `2`
    pub const fn ndim(&self) -> usize {
        N
    }
    pub fn new(x: Array1<T>, y: Array1<T>, f_xy: Array2<T>) -> Result<Self, ValidateError> {
        let data = Self {
            grid: [x, y],
            values: f_xy,
        };
        data.validate()?;
        Ok(data)
    }
}

/// 2-D interpolator
#[non_exhaustive]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Interp2D<T, S>
where
    T: Num + PartialOrd + Copy + Debug,
    S: Strategy2D<T>,
{
    pub data: InterpData2D<T>,
    pub strategy: S,
    #[cfg_attr(feature = "serde", serde(default))]
    pub extrapolate: Extrapolate<T>,
}

impl<T, S> Interp2D<T, S>
where
    T: Num + PartialOrd + Copy + Debug,
    S: Strategy2D<T>,
{
    /// Instantiate two-dimensional interpolator.
    ///
    /// Applicable interpolation strategies:
    /// - [`Linear`]
    /// - [`Nearest`]
    ///
    /// [`Extrapolate::Enable`] is valid for [`Linear`]
    ///
    /// # Example:
    /// ```
    /// use ndarray::prelude::*;
    /// use ninterp::prelude::*;
    /// // f(x, y) = 0.2 * x + 0.4 * y
    /// let interp = Interp2D::new(
    ///     // x
    ///     array![0., 1., 2.], // x0, x1, x2
    ///     // y
    ///     array![0., 1., 2.], // y0, y1, y2
    ///     // f(x, y)
    ///     array![
    ///         [0.0, 0.4, 0.8], // f(x0, y0), f(x0, y1), f(x0, y2)
    ///         [0.2, 0.6, 1.0], // f(x1, y0), f(x1, y1), f(x1, y2)
    ///         [0.4, 0.8, 1.2], // f(x2, y0), f(x2, y1), f(x2, y2)
    ///     ],
    ///     Linear,
    ///     Extrapolate::Clamp, // restrict point within grid bounds
    /// )
    /// .unwrap();
    /// assert_eq!(interp.interpolate(&[1.5, 1.5]).unwrap(), 0.9);
    /// assert_eq!(
    ///     interp.interpolate(&[-1., 2.5]).unwrap(),
    ///     interp.interpolate(&[0., 2.]).unwrap()
    /// ); // point is restricted to within grid bounds
    /// ```
    pub fn new(
        x: Array1<T>,
        y: Array1<T>,
        f_xy: Array2<T>,
        strategy: S,
        extrapolate: Extrapolate<T>,
    ) -> Result<Self, ValidateError> {
        let interpolator = Self {
            data: InterpData2D::new(x, y, f_xy)?,
            strategy,
            extrapolate,
        };
        interpolator.validate()?;
        Ok(interpolator)
    }

    pub fn set_extrapolate(&mut self, extrapolate: Extrapolate<T>) -> Result<(), ValidateError> {
        self.check_extrapolate(extrapolate)?;
        self.extrapolate = extrapolate;
        Ok(())
    }

    fn check_extrapolate(&self, extrapolate: Extrapolate<T>) -> Result<(), ValidateError> {
        // Check applicability of strategy and extrapolate setting
        if matches!(extrapolate, Extrapolate::Enable) && !self.strategy.allow_extrapolate() {
            return Err(ValidateError::ExtrapolateSelection(format!("{:?}", self.extrapolate)));
        }
        // If using Extrapolate::Enable,
        // check that each grid dimension has at least two elements
        if matches!(self.extrapolate, Extrapolate::Enable)
            && (self.data.grid[0].len() < 2 || self.data.grid[1].len() < 2)
        {
            return Err(ValidateError::Other(
                "at least 2 data points are required for extrapolation".into(),
            ));
        }
        Ok(())
    }
}

impl<T, S> Interpolator<T> for Interp2D<T, S>
where
    T: Num + PartialOrd + Copy + Debug,
    S: Strategy2D<T>,
{
    /// Returns `2`
    fn ndim(&self) -> usize {
        N
    }

    fn validate(&self) -> Result<(), ValidateError> {
        self.check_extrapolate(self.extrapolate)?;
        self.data.validate()?;
        Ok(())
    }

    fn interpolate(&self, point: &[T]) -> Result<T, InterpolateError> {
        let point: &[T; N] = point
            .try_into()
            .map_err(|_| InterpolateError::PointLength(N))?;
        let grid = [&self.data.grid[0], &self.data.grid[1]];
        let grid_names = ["x", "y"];
        let mut errors = Vec::new();
        for dim in 0..N {
            if !(grid[dim].first().unwrap()..=grid[dim].last().unwrap()).contains(&&point[dim]) {
                match self.extrapolate {
                    Extrapolate::Enable => {}
                    Extrapolate::Fill(value) => return Ok(value),
                    Extrapolate::Clamp => {
                        let clamped_point = &[
                            num::clamp(
                                point[0],
                                *self.data.grid[0].first().unwrap(),
                                *self.data.grid[0].last().unwrap(),
                            ),
                            num::clamp(
                                point[1],
                                *self.data.grid[1].first().unwrap(),
                                *self.data.grid[1].last().unwrap(),
                            ),
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

    fn extrapolate(&self) -> Option<Extrapolate<T>> {
        Some(self.extrapolate)
    }

    fn set_extrapolate(&mut self, extrapolate: Extrapolate<T>) -> Result<(), ValidateError> {
        self.check_extrapolate(extrapolate)?;
        self.extrapolate = extrapolate;
        Ok(())
    }
}

impl<T> Interp2D<T, Box<dyn Strategy2D<T>>>
where
    T: Num + PartialOrd + Copy + Debug,
{
    pub fn set_strategy(&mut self, strategy: Box<dyn Strategy2D<T>>) -> Result<(), ValidateError> {
        self.strategy = strategy;
        self.check_extrapolate(self.extrapolate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear() {
        let x = array![0.05, 0.10, 0.15];
        let y = array![0.10, 0.20, 0.30];
        let f_xy = array![[0., 1., 2.], [3., 4., 5.], [6., 7., 8.]];
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
                assert_eq!(interp.interpolate(&[*x_i, *y_j]).unwrap(), f_xy[[i, j]]);
            }
        }
        assert_eq!(interp.interpolate(&[x[2], y[1]]).unwrap(), f_xy[[2, 1]]);
        assert_eq!(interp.interpolate(&[0.075, 0.25]).unwrap(), 3.);
    }

    #[test]
    fn test_linear_offset() {
        let interp = Interp2D::new(
            array![0., 1.],
            array![0., 1.],
            array![[0., 1.], [2., 3.]],
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
            array![0.05, 0.10, 0.15],
            array![0.10, 0.20, 0.30],
            array![[0., 1., 2.], [3., 4., 5.], [6., 7., 8.]],
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
        let x = array![0.05, 0.10, 0.15];
        let y = array![0.10, 0.20, 0.30];
        let f_xy = array![[0., 1., 2.], [3., 4., 5.], [6., 7., 8.]];
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
                assert_eq!(interp.interpolate(&[*x_i, *y_j]).unwrap(), f_xy[[i, j]]);
            }
        }
        assert_eq!(interp.interpolate(&[0.05, 0.12]).unwrap(), f_xy[[0, 0]]);
        assert_eq!(
            // float imprecision
            interp.interpolate(&[0.07, 0.15 + 0.0001]).unwrap(),
            f_xy[[0, 1]]
        );
        assert_eq!(interp.interpolate(&[0.08, 0.21]).unwrap(), f_xy[[1, 1]]);
        assert_eq!(interp.interpolate(&[0.11, 0.26]).unwrap(), f_xy[[1, 2]]);
        assert_eq!(interp.interpolate(&[0.13, 0.12]).unwrap(), f_xy[[2, 0]]);
        assert_eq!(interp.interpolate(&[0.14, 0.29]).unwrap(), f_xy[[2, 2]]);
    }

    #[test]
    fn test_extrapolate_inputs() {
        // Extrapolate::Extrapolate
        assert!(matches!(
            Interp2D::new(
                array![0.1, 1.1],
                array![0.2, 1.2],
                array![[0., 1.], [2., 3.]],
                Nearest,
                Extrapolate::Enable,
            )
            .unwrap_err(),
            ValidateError::ExtrapolateSelection(_)
        ));
        // Extrapolate::Error
        let interp = Interp2D::new(
            array![0.1, 1.1],
            array![0.2, 1.2],
            array![[0., 1.], [2., 3.]],
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
            array![0.1, 1.1],
            array![0.2, 1.2],
            array![[0., 1.], [2., 3.]],
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
            array![0., 1.],
            array![0., 1.],
            array![[0., 1.], [2., 3.]],
            Box::new(Linear) as Box<dyn Strategy2D<_>>,
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
            array![0.1, 1.1],
            array![0.2, 1.2],
            array![[0., 1.], [2., 3.]],
            Linear,
            Extrapolate::Clamp,
        )
        .unwrap();
        assert_eq!(interp.interpolate(&[-1., -1.]).unwrap(), 0.);
        assert_eq!(interp.interpolate(&[2., 2.]).unwrap(), 3.);
    }
}
