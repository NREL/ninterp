//! 3-dimensional interpolation

use super::*;

mod strategies;

const N: usize = 3;

pub type InterpData3D<D> = InterpData<D, N>;
validate_impl!(InterpData3D<D>);
impl<D> InterpData3D<D>
where
    D: Data,
    D::Elem: Num + PartialOrd + Copy + Debug,
{
    /// Returns `3`
    pub const fn ndim(&self) -> usize {
        N
    }
    pub fn new(
        x: ArrayBase<D, Ix1>,
        y: ArrayBase<D, Ix1>,
        z: ArrayBase<D, Ix1>,
        f_xyz: ArrayBase<D, Ix3>,
    ) -> Result<Self, ValidateError> {
        let data = Self {
            grid: [x, y, z],
            values: f_xyz,
        };
        data.validate()?;
        Ok(data)
    }
}

/// 3-D interpolator
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
pub struct Interp3D<D, S>
where
    D: Data,
    D::Elem: Num + PartialOrd + Copy + Debug,
    S: Strategy3D<D>,
{
    pub data: InterpData3D<D>,
    pub strategy: S,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(
        feature = "serde",
        serde(bound = "D::Elem: Serialize + DeserializeOwned")
    )]
    pub extrapolate: Extrapolate<D::Elem>,
}

impl<D, S> Interp3D<D, S>
where
    D: Data,
    D::Elem: Num + PartialOrd + Copy + Debug,
    S: Strategy3D<D>,
{
    /// Instantiate three-dimensional interpolator.
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
    /// // f(x, y, z) = 0.2 * x + 0.2 * y + 0.2 * z
    /// let interp = Interp3D::new(
    ///     // x
    ///     array![1., 2.], // x0, x1
    ///     // y
    ///     array![1., 2.], // y0, y1
    ///     // z
    ///     array![1., 2.], // z0, z1
    ///     // f(x, y, z)
    ///     array![
    ///         [
    ///             [0.6, 0.8], // f(x0, y0, z0), f(x0, y0, z1)
    ///             [0.8, 1.0], // f(x0, y1, z0), f(x0, y1, z1)
    ///         ],
    ///         [
    ///             [0.8, 1.0], // f(x1, y0, z0), f(x1, y0, z1)
    ///             [1.0, 1.2], // f(x1, y1, z0), f(x1, y1, z1)
    ///         ],
    ///     ],
    ///     Linear,
    ///     Extrapolate::Error, // return an error when point is out of bounds
    /// )
    /// .unwrap();
    /// assert_eq!(interp.interpolate(&[1.5, 1.5, 1.5]).unwrap(), 0.9);
    /// // out of bounds point with `Extrapolate::Error` fails
    /// assert!(matches!(
    ///     interp.interpolate(&[2.5, 2.5, 2.5]).unwrap_err(),
    ///     ninterp::error::InterpolateError::ExtrapolateError(_)
    /// ));
    /// ```
    pub fn new(
        x: ArrayBase<D, Ix1>,
        y: ArrayBase<D, Ix1>,
        z: ArrayBase<D, Ix1>,
        f_xyz: ArrayBase<D, Ix3>,
        strategy: S,
        extrapolate: Extrapolate<D::Elem>,
    ) -> Result<Self, ValidateError> {
        let interpolator = Self {
            data: InterpData3D::new(x, y, z, f_xyz)?,
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
        if matches!(self.extrapolate, Extrapolate::Enable)
            && (self.data.grid[0].len() < 2
                || self.data.grid[1].len() < 2
                || self.data.grid[2].len() < 2)
        {
            return Err(ValidateError::Other(
                "at least 2 data points are required for extrapolation".into(),
            ));
        }
        Ok(())
    }
}

impl<D, S> Interpolator<D::Elem> for Interp3D<D, S>
where
    D: Data,
    D::Elem: Num + PartialOrd + Copy + Debug,
    S: Strategy3D<D>,
{
    /// Returns `3`
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
        let mut errors = Vec::new();
        for dim in 0..N {
            if !(self.data.grid[dim].first().unwrap()..=self.data.grid[dim].last().unwrap())
                .contains(&&point[dim])
            {
                match self.extrapolate {
                    Extrapolate::Enable => {}
                    Extrapolate::Fill(value) => return Ok(value),
                    Extrapolate::Clamp => {
                        let clamped_point = &[
                            clamp(
                                point[0],
                                *self.data.grid[0].first().unwrap(),
                                *self.data.grid[0].last().unwrap(),
                            ),
                            clamp(
                                point[1],
                                *self.data.grid[1].first().unwrap(),
                                *self.data.grid[1].last().unwrap(),
                            ),
                            clamp(
                                point[2],
                                *self.data.grid[2].first().unwrap(),
                                *self.data.grid[2].last().unwrap(),
                            ),
                        ];
                        return self.strategy.interpolate(&self.data, clamped_point);
                    }
                    Extrapolate::Error => {
                        errors.push(format!(
                            "\n    point[{dim}] = {:?} is out of bounds for grid dim {dim}= {:?}",
                            point[dim], self.data.grid[dim],
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

    fn extrapolate(&self) -> Option<Extrapolate<D::Elem>> {
        Some(self.extrapolate)
    }

    fn set_extrapolate(&mut self, extrapolate: Extrapolate<D::Elem>) -> Result<(), ValidateError> {
        self.check_extrapolate(&extrapolate)?;
        self.extrapolate = extrapolate;
        Ok(())
    }
}

impl<D> Interp3D<D, Box<dyn Strategy3D<D>>>
where
    D: Data,
    D::Elem: Num + PartialOrd + Copy + Debug,
{
    /// Update strategy dynamically.
    pub fn set_strategy(&mut self, strategy: Box<dyn Strategy3D<D>>) -> Result<(), ValidateError> {
        self.strategy = strategy;
        self.check_extrapolate(&self.extrapolate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear() {
        let x = array![0.05, 0.10, 0.15];
        let y = array![0.10, 0.20, 0.30];
        let z = array![0.20, 0.40, 0.60];
        let f_xyz = array![
            [[0., 1., 2.], [3., 4., 5.], [6., 7., 8.]],
            [[9., 10., 11.], [12., 13., 14.], [15., 16., 17.]],
            [[18., 19., 20.], [21., 22., 23.], [24., 25., 26.],],
        ];
        let interp = Interp3D::new(
            x.view(),
            y.view(),
            z.view(),
            f_xyz.view(),
            Linear,
            Extrapolate::Error,
        )
        .unwrap();
        // Check that interpolating at grid points just retrieves the value
        for (i, x_i) in x.iter().enumerate() {
            for (j, y_j) in y.iter().enumerate() {
                for (k, z_k) in z.iter().enumerate() {
                    assert_eq!(
                        interp.interpolate(&[*x_i, *y_j, *z_k]).unwrap(),
                        f_xyz[[i, j, k]]
                    );
                }
            }
        }
        assert_approx_eq!(interp.interpolate(&[x[0], y[0], 0.3]).unwrap(), 0.5);
        assert_approx_eq!(interp.interpolate(&[x[0], 0.15, z[0]]).unwrap(), 1.5);
        assert_approx_eq!(interp.interpolate(&[x[0], 0.15, 0.3]).unwrap(), 2.);
        assert_approx_eq!(interp.interpolate(&[0.075, y[0], z[0]]).unwrap(), 4.5);
        assert_approx_eq!(interp.interpolate(&[0.075, y[0], 0.3]).unwrap(), 5.);
        assert_approx_eq!(interp.interpolate(&[0.075, 0.15, z[0]]).unwrap(), 6.);
    }

    #[test]
    fn test_linear_extrapolation() {
        let interp = Interp3D::new(
            array![0.05, 0.10, 0.15],
            array![0.10, 0.20, 0.30],
            array![0.20, 0.40, 0.60],
            array![
                [[0., 1., 2.], [3., 4., 5.], [6., 7., 8.]],
                [[9., 10., 11.], [12., 13., 14.], [15., 16., 17.]],
                [[18., 19., 20.], [21., 22., 23.], [24., 25., 26.],],
            ],
            Linear,
            Extrapolate::Enable,
        )
        .unwrap();
        // below x, below y, below z
        assert_approx_eq!(interp.interpolate(&[0.01, 0.06, 0.17]).unwrap(), -8.55);
        assert_approx_eq!(interp.interpolate(&[0.02, 0.08, 0.19]).unwrap(), -6.05);
        // below x, below y, above z
        assert_approx_eq!(interp.interpolate(&[0.01, 0.06, 0.63]).unwrap(), -6.25);
        assert_approx_eq!(interp.interpolate(&[0.02, 0.08, 0.65]).unwrap(), -3.75);
        // below x, above y, below z
        assert_approx_eq!(interp.interpolate(&[0.01, 0.33, 0.17]).unwrap(), -0.45);
        assert_approx_eq!(interp.interpolate(&[0.02, 0.36, 0.19]).unwrap(), 2.35);
        // below x, above y, above z
        assert_approx_eq!(interp.interpolate(&[0.01, 0.33, 0.63]).unwrap(), 1.85);
        assert_approx_eq!(interp.interpolate(&[0.02, 0.36, 0.65]).unwrap(), 4.65);
        // above x, below y, below z
        assert_approx_eq!(interp.interpolate(&[0.17, 0.06, 0.17]).unwrap(), 20.25);
        assert_approx_eq!(interp.interpolate(&[0.19, 0.08, 0.19]).unwrap(), 24.55);
        // above x, below y, above z
        assert_approx_eq!(interp.interpolate(&[0.17, 0.06, 0.63]).unwrap(), 22.55);
        assert_approx_eq!(interp.interpolate(&[0.19, 0.08, 0.65]).unwrap(), 26.85);
        // above x, above y, below z
        assert_approx_eq!(interp.interpolate(&[0.17, 0.33, 0.17]).unwrap(), 28.35);
        assert_approx_eq!(interp.interpolate(&[0.19, 0.36, 0.19]).unwrap(), 32.95);
        // above x, above y, above z
        assert_approx_eq!(interp.interpolate(&[0.17, 0.33, 0.63]).unwrap(), 30.65);
        assert_approx_eq!(interp.interpolate(&[0.19, 0.36, 0.65]).unwrap(), 35.25);
    }

    #[test]
    fn test_linear_offset() {
        let interp = Interp3D::new(
            array![0., 1.],
            array![0., 1.],
            array![0., 1.],
            array![[[0., 1.], [2., 3.]], [[4., 5.], [6., 7.]],],
            Linear,
            Extrapolate::Error,
        )
        .unwrap();
        assert_approx_eq!(interp.interpolate(&[0.25, 0.65, 0.9]).unwrap(), 3.2);
    }

    #[test]
    fn test_nearest() {
        let x = array![0., 1.];
        let y = array![0., 1.];
        let z = array![0., 1.];
        let f_xyz = array![[[0., 1.], [2., 3.]], [[4., 5.], [6., 7.]],];
        let interp = Interp3D::new(
            x.view(),
            y.view(),
            z.view(),
            f_xyz.view(),
            Nearest,
            Extrapolate::Error,
        )
        .unwrap();
        // Check that interpolating at grid points just retrieves the value
        for (i, x_i) in x.iter().enumerate() {
            for (j, y_j) in y.iter().enumerate() {
                for (k, z_k) in z.iter().enumerate() {
                    assert_eq!(
                        interp.interpolate(&[*x_i, *y_j, *z_k]).unwrap(),
                        f_xyz[[i, j, k]]
                    );
                }
            }
        }
        assert_eq!(interp.interpolate(&[0., 0., 0.]).unwrap(), 0.);
        assert_eq!(interp.interpolate(&[0.25, 0.25, 0.25]).unwrap(), 0.);
        assert_eq!(interp.interpolate(&[0.25, 0.75, 0.25]).unwrap(), 2.);
        assert_eq!(interp.interpolate(&[0., 1., 0.]).unwrap(), 2.);
        assert_eq!(interp.interpolate(&[0.75, 0.25, 0.75]).unwrap(), 5.);
        assert_eq!(interp.interpolate(&[0.75, 0.75, 0.75]).unwrap(), 7.);
        assert_eq!(interp.interpolate(&[1., 1., 1.]).unwrap(), 7.);
    }

    #[test]
    fn test_extrapolate_inputs() {
        // Extrapolate::Extrapolate
        assert!(matches!(
            Interp3D::new(
                array![0.1, 1.1],
                array![0.2, 1.2],
                array![0.3, 1.3],
                array![[[0., 1.], [2., 3.]], [[4., 5.], [6., 7.]],],
                Nearest,
                Extrapolate::Enable,
            )
            .unwrap_err(),
            ValidateError::ExtrapolateSelection(_)
        ));
        // Extrapolate::Error
        let interp = Interp3D::new(
            array![0.1, 1.1],
            array![0.2, 1.2],
            array![0.3, 1.3],
            array![[[0., 1.], [2., 3.]], [[4., 5.], [6., 7.]],],
            Linear,
            Extrapolate::Error,
        )
        .unwrap();
        assert!(matches!(
            interp.interpolate(&[-1., -1., -1.]).unwrap_err(),
            InterpolateError::ExtrapolateError(_)
        ));
        assert!(matches!(
            interp.interpolate(&[2., 2., 2.]).unwrap_err(),
            InterpolateError::ExtrapolateError(_)
        ));
    }

    #[test]
    fn test_extrapolate_fill_value() {
        let interp = Interp3D::new(
            array![0.1, 1.1],
            array![0.2, 1.2],
            array![0.3, 1.3],
            array![[[0., 1.], [2., 3.]], [[4., 5.], [6., 7.]],],
            Linear,
            Extrapolate::Fill(f64::NAN),
        )
        .unwrap();
        assert_approx_eq!(interp.interpolate(&[0.4, 0.4, 0.4]).unwrap(), 1.7);
        assert_approx_eq!(interp.interpolate(&[0.8, 0.8, 0.8]).unwrap(), 4.5);
        assert!(interp.interpolate(&[0., 0., 0.]).unwrap().is_nan());
        assert!(interp.interpolate(&[0., 0., 2.]).unwrap().is_nan());
        assert!(interp.interpolate(&[0., 2., 0.]).unwrap().is_nan());
        assert!(interp.interpolate(&[0., 2., 2.]).unwrap().is_nan());
        assert!(interp.interpolate(&[2., 0., 0.]).unwrap().is_nan());
        assert!(interp.interpolate(&[2., 0., 2.]).unwrap().is_nan());
        assert!(interp.interpolate(&[2., 2., 0.]).unwrap().is_nan());
        assert!(interp.interpolate(&[2., 2., 2.]).unwrap().is_nan());
    }

    #[test]
    fn test_extrapolate_clamp() {
        let interp = Interp3D::new(
            array![0.1, 1.1],
            array![0.2, 1.2],
            array![0.3, 1.3],
            array![[[0., 1.], [2., 3.]], [[4., 5.], [6., 7.]],],
            Linear,
            Extrapolate::Clamp,
        )
        .unwrap();
        assert_eq!(interp.interpolate(&[-1., -1., -1.]).unwrap(), 0.);
        assert_eq!(interp.interpolate(&[2., 2., 2.]).unwrap(), 7.);
    }
}
