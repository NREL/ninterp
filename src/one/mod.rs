//! 1-dimensional interpolation

use super::*;

mod strategies;

const N: usize = 1;

pub type InterpData1D<D> = InterpData<D, N>;
validate_impl!(InterpData1D<D>);
impl<D> InterpData1D<D>
where
    D: Data,
    D::Elem: Num + PartialOrd + Copy + Debug,
{
    /// Returns `1`
    pub const fn ndim(&self) -> usize {
        N
    }
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
#[derive(Debug)]
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
    D: Data,
    D::Elem: Num + PartialOrd + Copy + Debug,
    S: Strategy1D<D>,
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

impl<D, S> Interp1D<D, S>
where
    D: Data,
    D::Elem: Num + PartialOrd + Copy + Debug,
    S: Strategy1D<D>,
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
    /// ); // point is restricted to within grid bounds
    /// ```
    pub fn new(
        x: ArrayBase<D, Ix1>,
        f_x: ArrayBase<D, Ix1>,
        strategy: S,
        extrapolate: Extrapolate<D::Elem>,
    ) -> Result<Self, ValidateError> {
        let interpolator = Self {
            data: InterpData1D::new(x, f_x)?,
            strategy,
            extrapolate,
        };
        interpolator.validate()?;
        Ok(interpolator)
    }

    fn check_extrapolate(&self, extrapolate: &Extrapolate<D::Elem>) -> Result<(), ValidateError> {
        // Check applicability of strategy and extrapolate setting
        if matches!(extrapolate, Extrapolate::Enable) && !self.strategy.allow_extrapolate() {
            return Err(ValidateError::ExtrapolateSelection(format!(
                "{:?}",
                self.extrapolate
            )));
        }
        // If using Extrapolate::Enable,
        // check that each grid dimension has at least two elements
        if matches!(self.extrapolate, Extrapolate::Enable) && self.data.grid[0].len() < 2 {
            return Err(ValidateError::Other(
                "at least 2 data points are required for extrapolation".into(),
            ));
        }
        Ok(())
    }
}

impl<D, S> Interpolator<D::Elem> for Interp1D<D, S>
where
    D: Data,
    D::Elem: Num + PartialOrd + Copy + Debug,
    S: Strategy1D<D>,
{
    /// Returns `1`
    fn ndim(&self) -> usize {
        N
    }

    fn validate(&self) -> Result<(), ValidateError> {
        self.check_extrapolate(&self.extrapolate)?;
        self.data.validate()?;
        Ok(())
    }

    fn interpolate(&self, point: &[D::Elem]) -> Result<D::Elem, InterpolateError> {
        let point: &[D::Elem; N] = point
            .try_into()
            .map_err(|_| InterpolateError::PointLength(N))?;
        if !(self.data.grid[0].first().unwrap()..=self.data.grid[0].last().unwrap())
            .contains(&&point[0])
        {
            match self.extrapolate {
                Extrapolate::Enable => {}
                Extrapolate::Fill(value) => return Ok(value),
                Extrapolate::Clamp => {
                    let clamped_point = &[clamp(
                        point[0],
                        *self.data.grid[0].first().unwrap(),
                        *self.data.grid[0].last().unwrap(),
                    )];
                    return self.strategy.interpolate(&self.data, clamped_point);
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

    fn extrapolate(&self) -> Option<Extrapolate<D::Elem>> {
        Some(self.extrapolate)
    }

    fn set_extrapolate(&mut self, extrapolate: Extrapolate<D::Elem>) -> Result<(), ValidateError> {
        self.check_extrapolate(&extrapolate)?;
        self.extrapolate = extrapolate;
        Ok(())
    }
}

impl<D> Interp1D<D, Box<dyn Strategy1D<D>>>
where
    D: Data,
    D::Elem: Num + PartialOrd + Copy + Debug,
{
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
        let interp = Interp1D::new(x.clone(), f_x.clone(), Linear, Extrapolate::Error).unwrap();
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
        let x = array![0., 1., 2., 3., 4.];
        let f_x = array![0.2, 0.4, 0.6, 0.8, 1.0];
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
        let x = array![0., 1., 2., 3., 4.];
        let f_x = array![0.2, 0.4, 0.6, 0.8, 1.0];
        let interp = Interp1D::new(x.clone(), f_x.clone(), Nearest, Extrapolate::Error).unwrap();
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
    fn test_extrapolate_fill_value() {
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
        assert_eq!(
            interp.interpolate(&[-0.75]).unwrap(),
            0.04999999999999999 // 0.05
        );
        assert_eq!(interp.interpolate(&[5.]).unwrap(), 1.2);
    }
}
