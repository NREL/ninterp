//! N-dimensional interpolation

use super::*;

use ndarray::prelude::*;

mod strategies;

#[non_exhaustive]
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct InterpND {
    pub grid: Vec<Vec<f64>>,
    pub values: ArrayD<f64>,
    pub strategy: Box<dyn StrategyND>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub extrapolate: Extrapolate,
}

impl InterpND {
    pub fn new(
        grid: Vec<Vec<f64>>,
        values: ArrayD<f64>,
        strategy: impl StrategyND + 'static,
        extrapolate: Extrapolate,
    ) -> Result<Self, ValidateError> {
        let interpolator = Self {
            grid,
            values,
            strategy: Box::new(strategy),
            extrapolate,
        };
        interpolator.validate()?;
        Ok(interpolator)
    }

    pub fn set_strategy(
        &mut self,
        strategy: impl StrategyND + 'static,
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
        for i in 0..self.ndim() {
            if matches!(self.extrapolate, Extrapolate::Enable) && self.grid[i].len() < 2 {
                return Err(ValidateError::Other(format!(
                    "at least 2 data points are required for extrapolation: dim {i}"
                )));
            }
        }
        Ok(())
    }
}

impl Interpolator for InterpND {
    fn ndim(&self) -> usize {
        if self.values.len() == 1 {
            0
        } else {
            self.values.ndim()
        }
    }

    fn validate(&self) -> Result<(), ValidateError> {
        self.check_extrapolate(self.extrapolate)?;
        for i in 0..self.ndim() {
            let i_grid_len = self.grid[i].len();

            // Check that each grid dimension has elements
            // Indexing `grid` directly is okay because empty dimensions are caught at compilation
            if i_grid_len == 0 {
                return Err(ValidateError::EmptyGrid(i.to_string()));
            }

            // Check that grid points are monotonically increasing
            if !self.grid[i].windows(2).all(|w| w[0] <= w[1]) {
                return Err(ValidateError::Monotonicity(i.to_string()));
            }

            // Check that grid and values are compatible shapes
            if i_grid_len != self.values.shape()[i] {
                return Err(ValidateError::IncompatibleShapes(i.to_string()));
            }
        }
        Ok(())
    }

    fn interpolate(&self, point: &[f64]) -> Result<f64, InterpolateError> {
        let n = self.ndim();
        if point.len() != n {
            return Err(InterpolateError::PointLength(n));
        }
        let mut errors = Vec::new();
        for dim in 0..n {
            if !(self.grid[dim].first().unwrap()..=self.grid[dim].last().unwrap())
                .contains(&&point[dim])
            {
                match self.extrapolate {
                    Extrapolate::Enable => {}
                    Extrapolate::Fill(value) => return Ok(value),
                    Extrapolate::Clamp => {
                        let clamped_point: Vec<f64> = point
                            .iter()
                            .enumerate()
                            .map(|(dim, pt)| {
                                pt.clamp(
                                    *self.grid[dim].first().unwrap(),
                                    *self.grid[dim].last().unwrap(),
                                )
                            })
                            .collect();
                        return self.strategy.interpolate(self, &clamped_point);
                    }
                    Extrapolate::Error => {
                        errors.push(format!(
                            "\n    point[{dim}] = {:?} is out of bounds for grid[{dim}] = {:?}",
                            point[dim], self.grid[dim],
                        ));
                    }
                };
            }
        }
        if !errors.is_empty() {
            return Err(InterpolateError::ExtrapolateError(errors.join("")));
        }
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
    fn test_linear() {
        let grid = vec![
            vec![0.05, 0.10, 0.15],
            vec![0.10, 0.20, 0.30],
            vec![0.20, 0.40, 0.60],
        ];
        let values = array![
            [[0., 1., 2.], [3., 4., 5.], [6., 7., 8.]],
            [[9., 10., 11.], [12., 13., 14.], [15., 16., 17.]],
            [[18., 19., 20.], [21., 22., 23.], [24., 25., 26.]],
        ]
        .into_dyn();
        let interp =
            InterpND::new(grid.clone(), values.clone(), Linear, Extrapolate::Error).unwrap();
        // Check that interpolating at grid points just retrieves the value
        for i in 0..grid[0].len() {
            for j in 0..grid[1].len() {
                for k in 0..grid[2].len() {
                    assert_eq!(
                        &interp
                            .interpolate(&[grid[0][i], grid[1][j], grid[2][k]])
                            .unwrap(),
                        values.slice(s![i, j, k]).first().unwrap()
                    );
                }
            }
        }
        assert_eq!(
            interp.interpolate(&[grid[0][0], grid[1][0], 0.3]).unwrap(),
            0.4999999999999999 // 0.5
        );
        assert_eq!(
            interp.interpolate(&[grid[0][0], 0.15, grid[2][0]]).unwrap(),
            1.4999999999999996 // 1.5
        );
        assert_eq!(
            interp.interpolate(&[grid[0][0], 0.15, 0.3]).unwrap(),
            1.9999999999999996 // 2.0
        );
        assert_eq!(
            interp
                .interpolate(&[0.075, grid[1][0], grid[2][0]])
                .unwrap(),
            4.499999999999999 // 4.5
        );
        assert_eq!(
            interp.interpolate(&[0.075, grid[1][0], 0.3]).unwrap(),
            4.999999999999999 // 5.0
        );
        assert_eq!(
            interp.interpolate(&[0.075, 0.15, grid[2][0]]).unwrap(),
            5.999999999999998 // 6.0
        );
    }

    #[test]
    fn test_linear_offset() {
        let interp = InterpND::new(
            vec![vec![0., 1.], vec![0., 1.], vec![0., 1.]],
            array![[[0., 1.], [2., 3.]], [[4., 5.], [6., 7.]],].into_dyn(),
            Linear,
            Extrapolate::Error,
        )
        .unwrap();
        assert_eq!(
            interp.interpolate(&[0.25, 0.65, 0.9]).unwrap(),
            3.1999999999999997
        ) // 3.2
    }

    #[test]
    fn test_linear_extrapolation_2d() {
        let interp_2d = crate::interpolator::Interp2D::new(
            vec![0.05, 0.10, 0.15],
            vec![0.10, 0.20, 0.30],
            vec![vec![0., 1., 2.], vec![3., 4., 5.], vec![6., 7., 8.]],
            Linear,
            Extrapolate::Enable,
        )
        .unwrap();
        let interp_nd = InterpND::new(
            vec![vec![0.05, 0.10, 0.15], vec![0.10, 0.20, 0.30]],
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
            vec![0.05, 0.10, 0.15],
            vec![0.10, 0.20, 0.30],
            vec![0.20, 0.40, 0.60],
            vec![
                vec![vec![0., 1., 2.], vec![3., 4., 5.], vec![6., 7., 8.]],
                vec![vec![9., 10., 11.], vec![12., 13., 14.], vec![15., 16., 17.]],
                vec![
                    vec![18., 19., 20.],
                    vec![21., 22., 23.],
                    vec![24., 25., 26.],
                ],
            ],
            Linear,
            Extrapolate::Enable,
        )
        .unwrap();
        let interp_nd = InterpND::new(
            vec![
                vec![0.05, 0.10, 0.15],
                vec![0.10, 0.20, 0.30],
                vec![0.20, 0.40, 0.60],
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
        let grid = vec![vec![0., 1.], vec![0., 1.], vec![0., 1.]];
        let values = array![[[0., 1.], [2., 3.]], [[4., 5.], [6., 7.]],].into_dyn();
        let interp =
            InterpND::new(grid.clone(), values.clone(), Nearest, Extrapolate::Error).unwrap();
        // Check that interpolating at grid points just retrieves the value
        for i in 0..grid[0].len() {
            for j in 0..grid[1].len() {
                for k in 0..grid[2].len() {
                    assert_eq!(
                        &interp
                            .interpolate(&[grid[0][i], grid[1][j], grid[2][k]])
                            .unwrap(),
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
                vec![vec![0., 1.], vec![0., 1.], vec![0., 1.]],
                array![[[0., 1.], [2., 3.]], [[4., 5.], [6., 7.]],].into_dyn(),
                Nearest,
                Extrapolate::Enable,
            )
            .unwrap_err(),
            ValidateError::ExtrapolateSelection(_)
        ));
        // Extrapolate::Error
        let interp = InterpND::new(
            vec![vec![0., 1.], vec![0., 1.], vec![0., 1.]],
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
            vec![vec![0.1, 1.1], vec![0.2, 1.2], vec![0.3, 1.3]],
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
            vec![vec![0.1, 1.1], vec![0.2, 1.2], vec![0.3, 1.3]],
            array![[[0., 1.], [2., 3.]], [[4., 5.], [6., 7.]],].into_dyn(),
            Linear,
            Extrapolate::Clamp,
        )
        .unwrap();
        assert_eq!(interp.interpolate(&[-1., -1., -1.]).unwrap(), 0.);
        assert_eq!(interp.interpolate(&[2., 2., 2.]).unwrap(), 7.);
    }
}
