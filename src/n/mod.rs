//! N-dimensional interpolation

use super::*;

use ndarray::prelude::*;

mod strategies;
/// Interpolator data where N is determined at runtime
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound = "
        D: DataOwned,
        D::Elem: Serialize + DeserializeOwned,
    ")
)]
pub struct InterpDataND<D>
where
    D: Data,
    D::Elem: Num + PartialOrd + Copy + Debug,
{
    pub grid: Vec<ArrayBase<D, Ix1>>,
    pub values: ArrayBase<D, IxDyn>,
}
validate_impl!(InterpDataND<D>);
impl<D> InterpDataND<D>
where
    D: Data,
    D::Elem: Num + PartialOrd + Copy + Debug,
{
    pub fn ndim(&self) -> usize {
        if self.values.len() == 1 {
            0
        } else {
            self.values.ndim()
        }
    }
    pub fn new(
        grid: Vec<ArrayBase<D, Ix1>>,
        values: ArrayBase<D, IxDyn>,
    ) -> Result<Self, ValidateError> {
        let data = Self { grid, values };
        data.validate()?;
        Ok(data)
    }
}

/// N-D interpolator
#[non_exhaustive]
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
pub struct InterpND<D, S>
where
    D: Data,
    D::Elem: Num + PartialOrd + Copy + Debug,
    S: StrategyND<D>,
{
    pub data: InterpDataND<D>,
    pub strategy: S,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(
        feature = "serde",
        serde(bound = "D::Elem: Serialize + DeserializeOwned")
    )]
    pub extrapolate: Extrapolate<D::Elem>,
}

impl<D, S> InterpND<D, S>
where
    D: Data,
    D::Elem: Num + PartialOrd + Copy + Debug,
    S: StrategyND<D>,
{
    /// Instantiate N-dimensional (any dimensionality) interpolator.
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
    /// let interp = InterpND::new(
    ///     // grid
    ///     vec![
    ///         array![1., 2.], // x0, x1
    ///         array![1., 2.], // y0, y1
    ///         array![1., 2.], // z0, z1
    ///     ],
    ///     // values
    ///     array![
    ///         [
    ///             [0.6, 0.8], // f(x0, y0, z0), f(x0, y0, z1)
    ///             [0.8, 1.0], // f(x0, y1, z0), f(x0, y1, z1)
    ///         ],
    ///         [
    ///             [0.8, 1.0], // f(x1, y0, z0), f(x1, y0, z1)
    ///             [1.0, 1.2], // f(x1, y1, z0), f(x1, y1, z1)
    ///         ],
    ///     ].into_dyn(),
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
        grid: Vec<ArrayBase<D, Ix1>>,
        values: ArrayBase<D, IxDyn>,
        strategy: S,
        extrapolate: Extrapolate<D::Elem>,
    ) -> Result<Self, ValidateError> {
        let interpolator = Self {
            data: InterpDataND::new(grid, values)?,
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
        for i in 0..self.ndim() {
            if matches!(self.extrapolate, Extrapolate::Enable) && self.data.grid[i].len() < 2 {
                return Err(ValidateError::Other(format!(
                    "at least 2 data points are required for extrapolation: dim {i}"
                )));
            }
        }
        Ok(())
    }
}

impl<D, S> Interpolator<D::Elem> for InterpND<D, S>
where
    D: Data,
    D::Elem: Num + PartialOrd + Copy + Debug,
    S: StrategyND<D>,
{
    fn ndim(&self) -> usize {
        self.data.ndim()
    }

    fn validate(&self) -> Result<(), ValidateError> {
        self.check_extrapolate(&self.extrapolate)?;
        self.data.validate()?;
        Ok(())
    }

    fn interpolate(&self, point: &[D::Elem]) -> Result<D::Elem, InterpolateError> {
        let n = self.ndim();
        if point.len() != n {
            return Err(InterpolateError::PointLength(n));
        }
        let mut errors = Vec::new();
        for dim in 0..n {
            if !(self.data.grid[dim].first().unwrap()..=self.data.grid[dim].last().unwrap())
                .contains(&&point[dim])
            {
                match &self.extrapolate {
                    Extrapolate::Enable => {}
                    Extrapolate::Fill(value) => return Ok(*value),
                    Extrapolate::Clamp => {
                        let clamped_point: Vec<_> = point
                            .iter()
                            .enumerate()
                            .map(|(dim, pt)| {
                                *clamp(
                                    pt,
                                    self.data.grid[dim].first().unwrap(),
                                    self.data.grid[dim].last().unwrap(),
                                )
                            })
                            .collect();
                        return self.strategy.interpolate(&self.data, &clamped_point);
                    }
                    Extrapolate::Error => {
                        errors.push(format!(
                            "\n    point[{dim}] = {:?} is out of bounds for grid[{dim}] = {:?}",
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

impl<D> InterpND<D, Box<dyn StrategyND<D>>>
where
    D: Data,
    D::Elem: Num + PartialOrd + Copy + Debug,
{
    /// Update strategy dynamically.
    pub fn set_strategy(&mut self, strategy: Box<dyn StrategyND<D>>) -> Result<(), ValidateError> {
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
        let grid = vec![x.view(), y.view(), z.view()];
        let values = array![
            [[0., 1., 2.], [3., 4., 5.], [6., 7., 8.]],
            [[9., 10., 11.], [12., 13., 14.], [15., 16., 17.]],
            [[18., 19., 20.], [21., 22., 23.], [24., 25., 26.]],
        ]
        .into_dyn();
        let interp = InterpND::new(grid, values.view(), Linear, Extrapolate::Error).unwrap();
        // Check that interpolating at grid points just retrieves the value
        for i in 0..x.len() {
            for j in 0..y.len() {
                for k in 0..z.len() {
                    assert_eq!(
                        &interp.interpolate(&[x[i], y[j], z[k]]).unwrap(),
                        values.slice(s![i, j, k]).first().unwrap()
                    );
                }
            }
        }
        assert_approx_eq!(interp.interpolate(&[x[0], y[0], 0.3]).unwrap(), 0.5);
        assert_approx_eq!(interp.interpolate(&[x[0], 0.15, z[0]]).unwrap(), 1.5);
        assert_approx_eq!(interp.interpolate(&[x[0], 0.15, 0.3]).unwrap(), 2.0);
        assert_approx_eq!(interp.interpolate(&[0.075, y[0], z[0]]).unwrap(), 4.5);
        assert_approx_eq!(interp.interpolate(&[0.075, y[0], 0.3]).unwrap(), 5.);
        assert_approx_eq!(interp.interpolate(&[0.075, 0.15, z[0]]).unwrap(), 6.);
    }

    #[test]
    fn test_linear_offset() {
        let interp = InterpND::new(
            vec![array![0., 1.], array![0., 1.], array![0., 1.]],
            array![[[0., 1.], [2., 3.]], [[4., 5.], [6., 7.]],].into_dyn(),
            Linear,
            Extrapolate::Error,
        )
        .unwrap();
        assert_approx_eq!(interp.interpolate(&[0.25, 0.65, 0.9]).unwrap(), 3.2)
    }

    #[test]
    fn test_linear_extrapolation_2d() {
        let interp_2d = crate::interpolator::Interp2D::new(
            array![0.05, 0.10, 0.15],
            array![0.10, 0.20, 0.30],
            array![[0., 1., 2.], [3., 4., 5.], [6., 7., 8.]],
            Linear,
            Extrapolate::Enable,
        )
        .unwrap();
        let interp_nd = InterpND::new(
            vec![array![0.05, 0.10, 0.15], array![0.10, 0.20, 0.30]],
            array![[0., 1., 2.], [3., 4., 5.], [6., 7., 8.]].into_dyn(),
            Linear,
            Extrapolate::Enable,
        )
        .unwrap();
        // below x, below y
        assert_eq!(
            interp_2d.interpolate(&[0.0, 0.0]).unwrap(),
            interp_nd.interpolate(&[0.0, 0.0]).unwrap()
        );
        assert_eq!(
            interp_2d.interpolate(&[0.03, 0.04]).unwrap(),
            interp_nd.interpolate(&[0.03, 0.04]).unwrap(),
        );
        // below x, above y
        assert_eq!(
            interp_2d.interpolate(&[0.0, 0.32]).unwrap(),
            interp_nd.interpolate(&[0.0, 0.32]).unwrap(),
        );
        assert_eq!(
            interp_2d.interpolate(&[0.03, 0.36]).unwrap(),
            interp_nd.interpolate(&[0.03, 0.36]).unwrap()
        );
        // above x, below y
        assert_eq!(
            interp_2d.interpolate(&[0.17, 0.0]).unwrap(),
            interp_nd.interpolate(&[0.17, 0.0]).unwrap(),
        );
        assert_eq!(
            interp_2d.interpolate(&[0.19, 0.04]).unwrap(),
            interp_nd.interpolate(&[0.19, 0.04]).unwrap(),
        );
        // above x, above y
        assert_eq!(
            interp_2d.interpolate(&[0.17, 0.32]).unwrap(),
            interp_nd.interpolate(&[0.17, 0.32]).unwrap()
        );
        assert_eq!(
            interp_2d.interpolate(&[0.19, 0.36]).unwrap(),
            interp_nd.interpolate(&[0.19, 0.36]).unwrap()
        );
    }

    #[test]
    fn test_linear_extrapolate_3d() {
        let interp_3d = crate::interpolator::Interp3D::new(
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
        let interp_nd = InterpND::new(
            vec![
                array![0.05, 0.10, 0.15],
                array![0.10, 0.20, 0.30],
                array![0.20, 0.40, 0.60],
            ],
            array![
                [[0., 1., 2.], [3., 4., 5.], [6., 7., 8.]],
                [[9., 10., 11.], [12., 13., 14.], [15., 16., 17.]],
                [[18., 19., 20.], [21., 22., 23.], [24., 25., 26.]],
            ]
            .into_dyn(),
            Linear,
            Extrapolate::Enable,
        )
        .unwrap();
        // below x, below y, below z
        assert_eq!(
            interp_3d.interpolate(&[0.01, 0.06, 0.17]).unwrap(),
            interp_nd.interpolate(&[0.01, 0.06, 0.17]).unwrap()
        );
        assert_eq!(
            interp_3d.interpolate(&[0.02, 0.08, 0.19]).unwrap(),
            interp_nd.interpolate(&[0.02, 0.08, 0.19]).unwrap()
        );
        // below x, below y, above z
        assert_eq!(
            interp_3d.interpolate(&[0.01, 0.06, 0.63]).unwrap(),
            interp_nd.interpolate(&[0.01, 0.06, 0.63]).unwrap()
        );
        assert_eq!(
            interp_3d.interpolate(&[0.02, 0.08, 0.65]).unwrap(),
            interp_nd.interpolate(&[0.02, 0.08, 0.65]).unwrap()
        );
        // below x, above y, below z
        assert_eq!(
            interp_3d.interpolate(&[0.01, 0.33, 0.17]).unwrap(),
            interp_nd.interpolate(&[0.01, 0.33, 0.17]).unwrap()
        );
        assert_eq!(
            interp_3d.interpolate(&[0.02, 0.36, 0.19]).unwrap(),
            interp_nd.interpolate(&[0.02, 0.36, 0.19]).unwrap()
        );
        // below x, above y, above z
        assert_eq!(
            interp_3d.interpolate(&[0.01, 0.33, 0.63]).unwrap(),
            interp_nd.interpolate(&[0.01, 0.33, 0.63]).unwrap()
        );
        assert_eq!(
            interp_3d.interpolate(&[0.02, 0.36, 0.65]).unwrap(),
            interp_nd.interpolate(&[0.02, 0.36, 0.65]).unwrap()
        );
        // above x, below y, below z
        assert_eq!(
            interp_3d.interpolate(&[0.17, 0.06, 0.17]).unwrap(),
            interp_nd.interpolate(&[0.17, 0.06, 0.17]).unwrap()
        );
        assert_eq!(
            interp_3d.interpolate(&[0.19, 0.08, 0.19]).unwrap(),
            interp_nd.interpolate(&[0.19, 0.08, 0.19]).unwrap()
        );
        // above x, below y, above z
        assert_eq!(
            interp_3d.interpolate(&[0.17, 0.06, 0.63]).unwrap(),
            interp_nd.interpolate(&[0.17, 0.06, 0.63]).unwrap()
        );
        assert_eq!(
            interp_3d.interpolate(&[0.19, 0.08, 0.65]).unwrap(),
            interp_nd.interpolate(&[0.19, 0.08, 0.65]).unwrap()
        );
        // above x, above y, below z
        assert_eq!(
            interp_3d.interpolate(&[0.17, 0.33, 0.17]).unwrap(),
            interp_nd.interpolate(&[0.17, 0.33, 0.17]).unwrap()
        );
        assert_eq!(
            interp_3d.interpolate(&[0.19, 0.36, 0.19]).unwrap(),
            interp_nd.interpolate(&[0.19, 0.36, 0.19]).unwrap()
        );
        // above x, above y, above z
        assert_eq!(
            interp_3d.interpolate(&[0.17, 0.33, 0.63]).unwrap(),
            interp_nd.interpolate(&[0.17, 0.33, 0.63]).unwrap()
        );
        assert_eq!(
            interp_3d.interpolate(&[0.19, 0.36, 0.65]).unwrap(),
            interp_nd.interpolate(&[0.19, 0.36, 0.65]).unwrap()
        );
    }

    #[test]
    fn test_nearest() {
        let x = array![0., 1.];
        let y = array![0., 1.];
        let z = array![0., 1.];
        let grid = vec![x.view(), y.view(), z.view()];
        let values = array![[[0., 1.], [2., 3.]], [[4., 5.], [6., 7.]],].into_dyn();
        let interp = InterpND::new(grid, values.view(), Nearest, Extrapolate::Error).unwrap();
        // Check that interpolating at grid points just retrieves the value
        for i in 0..x.len() {
            for j in 0..y.len() {
                for k in 0..z.len() {
                    assert_eq!(
                        &interp.interpolate(&[x[i], y[j], z[k]]).unwrap(),
                        values.slice(s![i, j, k]).first().unwrap()
                    );
                }
            }
        }
        assert_eq!(interp.interpolate(&[0.25, 0.25, 0.25]).unwrap(), 0.);
        assert_eq!(interp.interpolate(&[0.25, 0.75, 0.25]).unwrap(), 2.);
        assert_eq!(interp.interpolate(&[0.75, 0.25, 0.75]).unwrap(), 5.);
        assert_eq!(interp.interpolate(&[0.75, 0.75, 0.75]).unwrap(), 7.);
    }

    #[test]
    fn test_extrapolate_inputs() {
        // Extrapolate::Extrapolate
        assert!(matches!(
            InterpND::new(
                vec![array![0., 1.], array![0., 1.], array![0., 1.]],
                array![[[0., 1.], [2., 3.]], [[4., 5.], [6., 7.]],].into_dyn(),
                Nearest,
                Extrapolate::Enable,
            )
            .unwrap_err(),
            ValidateError::ExtrapolateSelection(_)
        ));
        // Extrapolate::Error
        let interp = InterpND::new(
            vec![array![0., 1.], array![0., 1.], array![0., 1.]],
            array![[[0., 1.], [2., 3.]], [[4., 5.], [6., 7.]],].into_dyn(),
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
        let interp = InterpND::new(
            vec![array![0.1, 1.1], array![0.2, 1.2], array![0.3, 1.3]],
            array![[[0., 1.], [2., 3.]], [[4., 5.], [6., 7.]],].into_dyn(),
            Linear,
            Extrapolate::Fill(f64::NAN),
        )
        .unwrap();
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
        let interp = InterpND::new(
            vec![array![0.1, 1.1], array![0.2, 1.2], array![0.3, 1.3]],
            array![[[0., 1.], [2., 3.]], [[4., 5.], [6., 7.]],].into_dyn(),
            Linear,
            Extrapolate::Clamp,
        )
        .unwrap();
        assert_eq!(interp.interpolate(&[-1., -1., -1.]).unwrap(), 0.);
        assert_eq!(interp.interpolate(&[2., 2., 2.]).unwrap(), 7.);
    }

    #[test]
    fn test_mismatched_grid() {
        assert!(matches!(
            InterpND::new(
                // 3-D grid
                vec![array![0., 1.], array![0., 1.], array![0., 1.]],
                // 2-D values
                array![[0., 1.], [2., 3.]].into_dyn(),
                Linear,
                Extrapolate::Error,
            )
            .unwrap_err(),
            ValidateError::Other(_)
        ));
        assert!(InterpND::new(
            vec![array![]],
            array![0.].into_dyn(),
            Linear,
            Extrapolate::Error,
        )
        .is_ok(),);
        assert!(matches!(
            InterpND::new(
                // non-empty grid
                vec![array![1.]],
                // 0-D values
                array![0.].into_dyn(),
                Linear,
                Extrapolate::Error,
            )
            .unwrap_err(),
            ValidateError::Other(_)
        ));
    }
}
