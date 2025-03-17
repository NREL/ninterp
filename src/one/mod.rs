//! 1-dimensional interpolation

use super::*;

mod strategies;

const N: usize = 1;

pub type InterpData1D<D> = InterpData<D, N>;
/// [`InterpData1D`] that views data.
pub type InterpData1DViewed<T> = InterpData1D<ndarray::ViewRepr<T>>;
/// [`InterpData1D`] that owns data.
pub type InterpData1DOwned<T> = InterpData1D<ndarray::OwnedRepr<T>>;

impl<D> InterpData1D<D>
where
    D: Data + RawDataClone + Clone,
    D::Elem: PartialOrd + Debug,
{
    pub fn new(x: ArrayBase<D, Ix1>, f_x: ArrayBase<D, Ix1>) -> Result<Self, ValidateError> {
        let data = Self {
            grid: [x],
            values: f_x,
        };
        data.validate()?;
        Ok(data)
    }
}

/// 1-D interpolator
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound = "
        D: DataOwned,
        D::Elem: Serialize + DeserializeOwned,
        S: Serialize + DeserializeOwned
    ")
)]
pub struct Interp1D<D, S>
where
    D: Data + RawDataClone + Clone,
    D::Elem: PartialEq + Debug,
    S: Strategy1D<D> + Clone,
{
    pub data: InterpData1D<D>,
    pub strategy: S,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(
        feature = "serde",
        serde(bound = "D::Elem: Serialize + DeserializeOwned")
    )]
    pub extrapolate: Extrapolate<D::Elem>,
}
/// [`Interp1D`] that views data.
pub type Interp1DViewed<T, S> = Interp1D<ndarray::ViewRepr<T>, S>;
/// [`Interp1D`] that owns data.
pub type Interp1DOwned<T, S> = Interp1D<ndarray::OwnedRepr<T>, S>;

extrapolate_impl!(Interp1D, Strategy1D);

impl<D, S> Interp1D<D, S>
where
    D: Data + RawDataClone + Clone,
    D::Elem: PartialOrd + Debug,
    S: Strategy1D<D> + Clone,
{
    /// Instantiate one-dimensional interpolator.
    ///
    /// Applicable interpolation strategies:
    /// - [`Linear`]
    /// - [`Nearest`]
    /// - [`LeftNearest`]
    /// - [`RightNearest`]
    ///
    /// [`Extrapolate::Enable`] is valid for [`Linear`]
    ///
    /// # Example:
    /// ```
    /// use ndarray::prelude::*;
    /// use ninterp::prelude::*;
    /// // f(x) = 0.4 * x
    /// let interp = Interp1D::new(
    ///     // x
    ///     array![0., 1., 2.], // x0, x1, x2
    ///     // f(x)
    ///     array![0.0, 0.4, 0.8], // f(x0), f(x1), f(x2)
    ///     Linear,
    ///     Extrapolate::Enable,
    /// )
    /// .unwrap();
    /// assert_eq!(interp.interpolate(&[1.4]).unwrap(), 0.56);
    /// assert_eq!(
    ///     interp.interpolate(&[3.6]).unwrap(),
    ///     1.44
    /// );
    /// ```
    pub fn new(
        x: ArrayBase<D, Ix1>,
        f_x: ArrayBase<D, Ix1>,
        strategy: S,
        extrapolate: Extrapolate<D::Elem>,
    ) -> Result<Self, ValidateError> {
        let mut interpolator = Self {
            data: InterpData1D::new(x, f_x)?,
            strategy,
            extrapolate,
        };
        interpolator.check_extrapolate(&interpolator.extrapolate)?;
        interpolator.strategy.init(&interpolator.data)?;
        Ok(interpolator)
    }
}

impl<D, S> Interpolator<D::Elem> for Interp1D<D, S>
where
    D: Data + RawDataClone + Clone,
    D::Elem: Num + Euclid + PartialOrd + Debug + Copy,
    S: Strategy1D<D> + Clone,
{
    /// Returns `1`.
    fn ndim(&self) -> usize {
        N
    }

    fn validate(&mut self) -> Result<(), ValidateError> {
        self.check_extrapolate(&self.extrapolate)?;
        self.data.validate()?;
        self.strategy.init(&self.data)?;
        Ok(())
    }

    fn interpolate(&self, point: &[D::Elem]) -> Result<D::Elem, InterpolateError> {
        let point: &[D::Elem; N] = point
            .try_into()
            .map_err(|_| InterpolateError::PointLength(N))?;
        if !(self.data.grid[0].first().unwrap()..=self.data.grid[0].last().unwrap())
            .contains(&&point[0])
        {
            match &self.extrapolate {
                Extrapolate::Enable => {}
                Extrapolate::Fill(value) => return Ok(value.clone()),
                Extrapolate::Clamp => {
                    let clamped_point = [*clamp(
                        &point[0],
                        self.data.grid[0].first().unwrap(),
                        self.data.grid[0].last().unwrap(),
                    )];
                    return self.strategy.interpolate(&self.data, &clamped_point);
                }
                Extrapolate::Wrap => {
                    let wrapped_point = [wrap(
                        point[0],
                        *self.data.grid[0].first().unwrap(),
                        *self.data.grid[0].last().unwrap(),
                    )];
                    return self.strategy.interpolate(&self.data, &wrapped_point);
                }
                Extrapolate::Error => {
                    return Err(InterpolateError::ExtrapolateError(format!(
                        "\n    point[0] = {:?} is out of bounds for grid dim 0 = {:?}",
                        point[0], self.data.grid[0]
                    )))
                }
            }
        };
        self.strategy.interpolate(&self.data, point)
    }
}

impl<D> Interp1D<D, Box<dyn Strategy1D<D>>>
where
    D: Data + RawDataClone + Clone,
    D::Elem: PartialEq + Debug,
{
    /// Update strategy dynamically.
    pub fn set_strategy(&mut self, strategy: Box<dyn Strategy1D<D>>) -> Result<(), ValidateError> {
        self.strategy = strategy;
        self.check_extrapolate(&self.extrapolate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_args() {
        let interp = Interp1D::new(
            array![0., 1., 2., 3., 4.],
            array![0.2, 0.4, 0.6, 0.8, 1.0],
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
        let x = array![0., 1., 2., 3., 4.];
        let f_x = array![0.2, 0.4, 0.6, 0.8, 1.0];
        let interp = Interp1D::new(x.view(), f_x.view(), Linear, Extrapolate::Error).unwrap();
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
        let x = array![0., 1., 2., 3., 4.];
        let f_x = array![0.2, 0.4, 0.6, 0.8, 1.0];
        let interp = Interp1D::new(x.view(), f_x.view(), LeftNearest, Extrapolate::Error).unwrap();
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
        let x = array![0., 1., 2., 3., 4.];
        let f_x = array![0.2, 0.4, 0.6, 0.8, 1.0];
        let interp = Interp1D::new(x.view(), f_x.view(), RightNearest, Extrapolate::Error).unwrap();
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
        let x = array![0., 1., 2., 3., 4.];
        let f_x = array![0.2, 0.4, 0.6, 0.8, 1.0];
        let interp = Interp1D::new(x.view(), f_x.view(), Nearest, Extrapolate::Error).unwrap();
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
                array![0., 1., 2., 3., 4.],
                array![0.2, 0.4, 0.6, 0.8, 1.0],
                Nearest,
                Extrapolate::Enable,
            )
            .unwrap_err(),
            ValidateError::ExtrapolateSelection(_)
        ));

        // Extrapolate::Error
        let interp = Interp1D::new(
            array![0., 1., 2., 3., 4.],
            array![0.2, 0.4, 0.6, 0.8, 1.0],
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
    fn test_extrapolate_fill() {
        let interp = Interp1D::new(
            array![0., 1., 2., 3., 4.],
            array![0.2, 0.4, 0.6, 0.8, 1.0],
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
            array![0., 1., 2., 3., 4.],
            array![0.2, 0.4, 0.6, 0.8, 1.0],
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
            array![0., 1., 2., 3., 4.],
            array![0.2, 0.4, 0.6, 0.8, 1.0],
            Linear,
            Extrapolate::Enable,
        )
        .unwrap();
        assert_eq!(interp.interpolate(&[-1.]).unwrap(), 0.0);
        assert_approx_eq!(interp.interpolate(&[-0.75]).unwrap(), 0.05);
        assert_eq!(interp.interpolate(&[5.]).unwrap(), 1.2);
    }

    #[test]
    fn test_cubic_natural() {
        let x = array![1., 2., 3., 5., 7., 8.];
        let f_x = array![3., 6., 19., 99., 291., 444.];

        let interp =
            Interp1D::new(x.view(), f_x.view(), Cubic::natural(), Extrapolate::Enable).unwrap();

        // Interpolating at knots returns values
        for i in 0..x.len() {
            assert_approx_eq!(interp.interpolate(&[x[i]]).unwrap(), f_x[i]);
        }

        let x0 = x.first().unwrap();
        let xn = x.last().unwrap();
        let y0 = f_x.first().unwrap();
        let yn = f_x.last().unwrap();

        let range = xn - x0;

        let x_low = x0 - 0.2 * range;
        let y_low = interp.interpolate(&[x_low]).unwrap();
        let slope_low = (y0 - y_low) / (x0 - x_low);

        let x_high = xn + 0.2 * range;
        let y_high = interp.interpolate(&[x_high]).unwrap();
        let slope_high = (y_high - yn) / (x_high - xn);

        let xs_left = Array1::linspace(*x0, x0 + 2e-6, 50);
        let xs_right = Array1::linspace(xn - 2e-6, *xn, 50);

        // Left extrapolation is linear
        let ys: Array1<f64> = xs_left
            .iter()
            .map(|&x| interp.interpolate(&[x]).unwrap())
            .collect();
        let slopes: Array1<f64> = xs_left
            .windows(2)
            .into_iter()
            .zip(ys.windows(2))
            .map(|(x, y)| (y[1] - y[0]) / (x[1] - x[0]))
            .collect();
        assert_approx_eq!(slopes.mean().unwrap(), slope_low);

        // Right extrapolation is linear
        let ys: Array1<f64> = xs_right
            .iter()
            .map(|&x| interp.interpolate(&[x]).unwrap())
            .collect();
        let slopes: Array1<f64> = xs_right
            .windows(2)
            .into_iter()
            .zip(ys.windows(2))
            .map(|(x, y)| (y[1] - y[0]) / (x[1] - x[0]))
            .collect();
        assert_approx_eq!(slopes.mean().unwrap(), slope_high);
    }

    #[test]
    fn test_cubic_clamped() {
        let x = array![1., 2., 3., 5., 7., 8.];
        let f_x = array![3., -90., 19., 99., 291., 444.];

        let xs_left = Array1::linspace(x.first().unwrap() - 1e-6, x.first().unwrap() + 1e-6, 50);
        let xs_right = Array1::linspace(x.last().unwrap() - 1e-6, x.last().unwrap() + 1e-6, 50);

        for (a, b) in [(-5., 10.), (0., 0.), (2.4, -5.2)] {
            let interp = Interp1D::new(
                x.view(),
                f_x.view(),
                Cubic::clamped(a, b),
                Extrapolate::Enable,
            )
            .unwrap();

            // Interpolating at knots returns values
            for i in 0..x.len() {
                assert_approx_eq!(interp.interpolate(&[x[i]]).unwrap(), f_x[i]);
            }

            // Left slope = a
            let ys: Array1<f64> = xs_left
                .iter()
                .map(|&x| interp.interpolate(&[x]).unwrap())
                .collect();
            let slopes: Array1<f64> = xs_left
                .windows(2)
                .into_iter()
                .zip(ys.windows(2))
                .map(|(x, y)| (y[1] - y[0]) / (x[1] - x[0]))
                .collect();
            assert_approx_eq!(slopes.mean().unwrap(), a);

            // Right slope = b
            let ys: Array1<f64> = xs_right
                .iter()
                .map(|&x| interp.interpolate(&[x]).unwrap())
                .collect();
            let slopes: Array1<f64> = xs_right
                .windows(2)
                .into_iter()
                .zip(ys.windows(2))
                .map(|(x, y)| (y[1] - y[0]) / (x[1] - x[0]))
                .collect();
            assert_approx_eq!(slopes.mean().unwrap(), b);
        }
    }

    #[test]
    fn test_cubic_periodic() {
        let x = array![1., 2., 3., 5., 7., 8.];
        let f_x = array![3., -90., 19., 99., 291., 444.];

        let interp_extrap_enable =
            Interp1D::new(x.view(), f_x.view(), Cubic::periodic(), Extrapolate::Enable).unwrap();
        let interp_extrap_wrap =
            Interp1D::new(x.view(), f_x.view(), Cubic::periodic(), Extrapolate::Wrap).unwrap();

        // Interpolating at knots returns values
        for i in 0..x.len() {
            assert_approx_eq!(interp_extrap_enable.interpolate(&[x[i]]).unwrap(), f_x[i]);
            assert_approx_eq!(interp_extrap_wrap.interpolate(&[x[i]]).unwrap(), f_x[i]);
        }

        // Extrapolate::Enable is equivalent to Extrapolate::Wrap for Cubic::periodic()
        let x0 = x.first().unwrap();
        let xn = x.last().unwrap();
        let range = xn - x0;
        let x_low = x0 - 0.2 * range;
        let x_high = x0 + 0.2 * range;
        let xs_left = Array1::linspace(x_low, *x0, 50);
        let xs_right = Array1::linspace(*xn, x_high, 50);
        for x in xs_left {
            assert_eq!(
                interp_extrap_enable.interpolate(&[x]).unwrap(),
                interp_extrap_wrap.interpolate(&[x]).unwrap()
            );
        }
        for x in xs_right {
            assert_eq!(
                interp_extrap_enable.interpolate(&[x]).unwrap(),
                interp_extrap_wrap.interpolate(&[x]).unwrap()
            );
        }

        // Slope left
        let xs_left = Array1::linspace(x_low, x_low + 2e6, 50);
        let ys_left: Array1<f64> = xs_left
            .iter()
            .map(|&x| interp_extrap_enable.interpolate(&[x]).unwrap())
            .collect();
        let slopes_left: Array1<f64> = xs_left
            .windows(2)
            .into_iter()
            .zip(ys_left.windows(2))
            .map(|(x, y)| (y[1] - y[0]) / (x[1] - x[0]))
            .collect();
        let slope_left = slopes_left.mean().unwrap();
        // Slope right
        let xs_right = Array1::linspace(x_high - 2e6, x_high, 50);
        let ys_right: Array1<f64> = xs_right
            .iter()
            .map(|&x| interp_extrap_enable.interpolate(&[x]).unwrap())
            .collect();
        let slopes_right: Array1<f64> = xs_right
            .windows(2)
            .into_iter()
            .zip(ys_right.windows(2))
            .map(|(x, y)| (y[1] - y[0]) / (x[1] - x[0]))
            .collect();
        let slope_right = slopes_right.mean().unwrap();
        // Slopes at left and right are equal
        assert_approx_eq!(slope_left, slope_right);
    }
}
