//! 2-dimensional interpolation

use super::*;

mod strategies;
#[cfg(test)]
mod tests;

const N: usize = 2;

pub type InterpData2D<D> = InterpData<D, N>;
/// [`InterpData2D`] that views data.
pub type InterpData2DViewed<T> = InterpData2D<ndarray::ViewRepr<T>>;
/// [`InterpData2D`] that owns data.
pub type InterpData2DOwned<T> = InterpData2D<ndarray::OwnedRepr<T>>;

impl<D> InterpData2D<D>
where
    D: Data + RawDataClone + Clone,
    D::Elem: PartialOrd + Debug,
{
    pub fn new(
        x: ArrayBase<D, Ix1>,
        y: ArrayBase<D, Ix1>,
        f_xy: ArrayBase<D, Ix2>,
    ) -> Result<Self, ValidateError> {
        let data = Self {
            grid: [x, y],
            values: f_xy,
        };
        data.validate()?;
        Ok(data)
    }
}

/// 2-D interpolator
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound = "
        D: DataOwned,
        D::Elem: Serialize + DeserializeOwned,
        S: Serialize + DeserializeOwned
    ")
)]
pub struct Interp2D<D, S>
where
    D: Data + RawDataClone + Clone,
    D::Elem: PartialEq + Debug,
    S: Strategy2D<D> + Clone,
{
    pub data: InterpData2D<D>,
    pub strategy: S,
    #[cfg_attr(feature = "serde", serde(default))]
    pub extrapolate: Extrapolate<D::Elem>,
}
/// [`Interp2D`] that views data.
pub type Interp2DViewed<T, S> = Interp2D<ndarray::ViewRepr<T>, S>;
/// [`Interp2D`] that owns data.
pub type Interp2DOwned<T, S> = Interp2D<ndarray::OwnedRepr<T>, S>;

extrapolate_impl!(Interp2D, Strategy2D);
partialeq_impl!(Interp2D, InterpData2D, Strategy2D);

impl<D, S> Interp2D<D, S>
where
    D: Data + RawDataClone + Clone,
    D::Elem: PartialOrd + Debug,
    S: Strategy2D<D> + Clone,
{
    /// Instantiate two-dimensional interpolator.
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
    ///     strategy::Linear,
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
        x: ArrayBase<D, Ix1>,
        y: ArrayBase<D, Ix1>,
        f_xy: ArrayBase<D, Ix2>,
        strategy: S,
        extrapolate: Extrapolate<D::Elem>,
    ) -> Result<Self, ValidateError> {
        let mut interpolator = Self {
            data: InterpData2D::new(x, y, f_xy)?,
            strategy,
            extrapolate,
        };
        interpolator.check_extrapolate(&interpolator.extrapolate)?;
        interpolator.strategy.init(&interpolator.data)?;
        Ok(interpolator)
    }
}

impl<D, S> Interpolator<D::Elem> for Interp2D<D, S>
where
    D: Data + RawDataClone + Clone,
    D::Elem: Num + Euclid + PartialOrd + Debug + Copy,
    S: Strategy2D<D> + Clone,
{
    /// Returns `2`.
    #[inline]
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
        let mut errors = Vec::new();
        for dim in 0..N {
            if !(self.data.grid[dim].first().unwrap()..=self.data.grid[dim].last().unwrap())
                .contains(&&point[dim])
            {
                match &self.extrapolate {
                    Extrapolate::Enable => {}
                    Extrapolate::Fill(value) => return Ok(*value),
                    Extrapolate::Clamp => {
                        let clamped_point = std::array::from_fn(|i| {
                            *clamp(
                                &point[i],
                                self.data.grid[i].first().unwrap(),
                                self.data.grid[i].last().unwrap(),
                            )
                        });
                        return self.strategy.interpolate(&self.data, &clamped_point);
                    }
                    Extrapolate::Wrap => {
                        let wrapped_point = std::array::from_fn(|i| {
                            wrap(
                                point[i],
                                *self.data.grid[i].first().unwrap(),
                                *self.data.grid[i].last().unwrap(),
                            )
                        });
                        return self.strategy.interpolate(&self.data, &wrapped_point);
                    }
                    Extrapolate::Error => {
                        errors.push(format!(
                            "\n    point[{dim}] = {:?} is out of bounds for grid dim {dim} = {:?}",
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

impl<D> Interp2D<D, Box<dyn Strategy2D<D>>>
where
    D: Data + RawDataClone + Clone,
    D::Elem: PartialEq + Debug,
{
    /// Update strategy dynamically.
    pub fn set_strategy(&mut self, strategy: Box<dyn Strategy2D<D>>) -> Result<(), ValidateError> {
        self.strategy = strategy;
        self.check_extrapolate(&self.extrapolate)
    }
}

impl<D> Interp2D<D, strategy::enums::Strategy2DEnum>
where
    D: Data + RawDataClone + Clone,
    D::Elem: Num + PartialOrd + Copy + Debug,
{
    /// Update strategy dynamically.
    pub fn set_strategy(
        &mut self,
        strategy: impl Into<strategy::enums::Strategy2DEnum>,
    ) -> Result<(), ValidateError> {
        self.strategy = strategy.into();
        self.check_extrapolate(&self.extrapolate)
    }
}
