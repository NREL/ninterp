//! N-dimensional interpolation

use super::*;

use ndarray::prelude::*;

mod strategies;
#[cfg(test)]
mod tests;

/// Interpolator data where N is determined at runtime
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound(
        serialize = "D::Elem: Serialize",
        deserialize = "
            D: DataOwned,
            D::Elem: Deserialize<'de>,
        "
    ))
)]
pub struct InterpDataND<D>
where
    D: Data + RawDataClone + Clone,
    D::Elem: PartialEq + Debug,
{
    pub grid: Vec<ArrayBase<D, Ix1>>,
    pub values: ArrayBase<D, IxDyn>,
}
/// [`InterpDataND`] that views data.
pub type InterpDataNDViewed<T> = InterpDataND<ndarray::ViewRepr<T>>;
/// [`InterpDataND`] that owns data.
pub type InterpDataNDOwned<T> = InterpDataND<ndarray::OwnedRepr<T>>;

impl<D> PartialEq for InterpDataND<D>
where
    D: Data + RawDataClone + Clone,
    D::Elem: PartialEq + Debug,
    ArrayBase<D, Ix1>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.grid == other.grid && self.values == other.values
    }
}

impl<D> InterpDataND<D>
where
    D: Data + RawDataClone + Clone,
    D::Elem: PartialOrd + Debug,
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

    pub fn validate(&self) -> Result<(), ValidateError> {
        let n = self.ndim();
        if (self.grid.len() != n) && !(n == 0 && self.grid.iter().all(|g| g.is_empty())) {
            // Only possible for `InterpDataND`
            return Err(ValidateError::Other(format!(
                "grid length {} does not match dimensionality {}",
                self.grid.len(),
                n,
            )));
        }
        for i in 0..n {
            let i_grid_len = self.grid[i].len();
            // Check that each grid dimension has elements
            // Indexing `grid` directly is okay because empty dimensions are caught at compilation
            if i_grid_len == 0 {
                return Err(ValidateError::EmptyGrid(i));
            }
            // Check that grid points are monotonically increasing
            if !self.grid[i].windows(2).into_iter().all(|w| w[0] <= w[1]) {
                return Err(ValidateError::Monotonicity(i));
            }
            // Check that grid and values are compatible shapes
            if i_grid_len != self.values.shape()[i] {
                return Err(ValidateError::IncompatibleShapes(i));
            }
        }
        Ok(())
    }
}

/// N-D interpolator
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound(
        serialize = "
            D::Elem: Serialize,
            S: Serialize,
        ",
        deserialize = "
            D: DataOwned,
            D::Elem: Deserialize<'de>,
            S: Deserialize<'de>
        "
    ))
)]
pub struct InterpND<D, S>
where
    D: Data + RawDataClone + Clone,
    D::Elem: PartialEq + Debug,
    S: StrategyND<D> + Clone,
{
    pub data: InterpDataND<D>,
    pub strategy: S,
    #[cfg_attr(feature = "serde", serde(default))]
    pub extrapolate: Extrapolate<D::Elem>,
}
/// [`InterpND`] that views data.
pub type InterpNDViewed<T, S> = InterpND<ndarray::ViewRepr<T>, S>;
/// [`InterpND`] that owns data.
pub type InterpNDOwned<T, S> = InterpND<ndarray::OwnedRepr<T>, S>;

extrapolate_impl!(InterpND, StrategyND);
partialeq_impl!(InterpND, InterpDataND, StrategyND);

impl<D, S> InterpND<D, S>
where
    D: Data + RawDataClone + Clone,
    D::Elem: PartialOrd + Debug,
    S: StrategyND<D> + Clone,
{
    /// Instantiate N-dimensional (any dimensionality) interpolator.
    ///
    /// Applicable interpolation strategies:
    /// - [`strategy::Linear`]
    /// - [`strategy::Nearest`]
    ///
    /// [`Extrapolate::Enable`] is valid for [`strategy::Linear`]
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
    ///     strategy::Linear,
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
        let mut interpolator = Self {
            data: InterpDataND::new(grid, values)?,
            strategy,
            extrapolate,
        };
        interpolator.check_extrapolate(&interpolator.extrapolate)?;
        interpolator.strategy.init(&interpolator.data)?;
        Ok(interpolator)
    }
}

impl<D, S> Interpolator<D::Elem> for InterpND<D, S>
where
    D: Data + RawDataClone + Clone,
    D::Elem: Num + Euclid + PartialOrd + Debug + Copy,
    S: StrategyND<D> + Clone,
{
    #[inline]
    fn ndim(&self) -> usize {
        self.data.ndim()
    }

    fn validate(&mut self) -> Result<(), ValidateError> {
        self.check_extrapolate(&self.extrapolate)?;
        self.data.validate()?;
        self.strategy.init(&self.data)?;
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
                    Extrapolate::Wrap => {
                        let wrapped_point: Vec<_> = point
                            .iter()
                            .enumerate()
                            .map(|(dim, pt)| {
                                wrap(
                                    *pt,
                                    *self.data.grid[dim].first().unwrap(),
                                    *self.data.grid[dim].last().unwrap(),
                                )
                            })
                            .collect();
                        return self.strategy.interpolate(&self.data, &wrapped_point);
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

    fn set_extrapolate(&mut self, extrapolate: Extrapolate<D::Elem>) -> Result<(), ValidateError> {
        self.check_extrapolate(&extrapolate)?;
        self.extrapolate = extrapolate;
        Ok(())
    }
}

impl<D> InterpND<D, Box<dyn StrategyND<D>>>
where
    D: Data + RawDataClone + Clone,
    D::Elem: PartialEq + Debug,
{
    /// Update strategy dynamically.
    pub fn set_strategy(&mut self, strategy: Box<dyn StrategyND<D>>) -> Result<(), ValidateError> {
        self.strategy = strategy;
        self.check_extrapolate(&self.extrapolate)
    }
}

impl<D> InterpND<D, strategy::enums::StrategyNDEnum>
where
    D: Data + RawDataClone + Clone,
    D::Elem: Num + PartialOrd + Copy + Debug,
{
    /// Update strategy dynamically.
    pub fn set_strategy(
        &mut self,
        strategy: impl Into<strategy::enums::StrategyNDEnum>,
    ) -> Result<(), ValidateError> {
        self.strategy = strategy.into();
        self.check_extrapolate(&self.extrapolate)
    }
}
