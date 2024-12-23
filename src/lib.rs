//! The `ninterp` crate provides
//! [multivariate interpolation](https://en.wikipedia.org/wiki/Multivariate_interpolation#Regular_grid)
//!  over a regular, sorted, nonrepeating grid of any dimensionality.
//! A variety of interpolation strategies are implemented, however more are likely to be added.
//! Extrapolation beyond the range of the supplied coordinates
//! is supported for 1-D linear interpolators, using the slope of the nearby points.
//!
//! There are hard-coded interpolators for lower dimensionalities (up to N = 3) for better runtime performance.
//!
//! All interpolation is handled through instances of the [`Interpolator`] enum,
//! with the selected tuple variant containing relevant data.
//! Interpolation is executed by calling [`Interpolator::interpolate`].
//!
//! # Feature Flags
//! - `serde`: support for [`serde`](https://crates.io/crates/serde)
//!
//! # Getting Started
//! See the [`Interpolator`] enum documentation for examples and notes on usage.
//!

pub mod error;
pub mod n;
pub mod one;
pub mod three;
pub mod two;

pub use error::*;
pub use n::*;
pub use one::*;
pub use three::*;
pub use two::*;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// This method contains code from RouteE Compass, another open-source NREL-developed tool
// <https://www.nrel.gov/transportation/route-energy-prediction-model.html>
// <https://github.com/NREL/routee-compass/>
fn find_nearest_index(arr: &[f64], target: f64) -> usize {
    if &target == arr.last().unwrap() {
        return arr.len() - 2;
    }

    let mut low = 0;
    let mut high = arr.len() - 1;

    while low < high {
        let mid = low + (high - low) / 2;

        if arr[mid] >= target {
            high = mid;
        } else {
            low = mid + 1;
        }
    }

    if low > 0 && arr[low] >= target {
        low - 1
    } else {
        low
    }
}

/// An interpolator, with different functionality depending on variant.
///
/// Interpolation is executed by calling [`Interpolator::interpolate`].
/// The length of the supplied point slice must be equal to the intepolator dimensionality.
///
/// # Note
/// With interpolators of dimensionality N ≥ 1:
/// - By design, instantiation must be done via the interpolator structs `new` method.
/// This ensures that a validation check is performed to catch any potential errors early.
///   - To set or get field values, use the corresponding named methods.
/// - An interpolation [`Strategy`] (e.g. linear, left-nearest, etc.) must be specified.
/// Not all interpolation strategies are implemented for every dimensionality.
/// [`Strategy::Linear`] is implemented for all dimensionalities.
/// - An [`Extrapolate`] setting must be specified.
/// This controls what happens when a point is beyond the range of supplied coordinates.
/// If you are unsure which variant to choose, [`Extrapolate::Error`] is likely what you want.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum Interpolator {
    /// Useful for returning a constant-value result from an interpolator.
    ///
    /// # Example:
    /// ```
    /// use ninterp::*;
    /// // 0-D is unique, the value is directly provided in the variant
    /// let const_value = 0.5;
    /// let interp = Interpolator::Interp0D(const_value);
    /// assert_eq!(
    ///     interp.interpolate(&[]).unwrap(), // an empty point is required for 0-D
    ///     const_value
    /// );
    /// ```
    Interp0D(f64),
    /// Interpolates in one dimension. Contains an [`Interp1D`] instance defining x and f(x).
    ///
    /// Applicable interpolation strategies:
    /// - [`Strategy::Linear`]
    /// - [`Strategy::LeftNearest`]
    /// - [`Strategy::RightNearest`]
    /// - [`Strategy::Nearest`]
    ///
    /// # Example (linear, using [`Extrapolate::Enable`]):
    /// ```
    /// use ninterp::*;
    /// let interp = Interpolator::Interp1D(
    ///     // f(x) = 0.2 * x + 0.2
    ///     Interp1D::new(
    ///         vec![0., 1., 2.], // x0, x1, x2
    ///         vec![0.2, 0.4, 0.6], // f(x0), f(x1), f(x2)
    ///         Strategy::Linear, // linear interpolation
    ///         Extrapolate::Enable, // linearly extrapolate when point is out of bounds
    ///     )
    ///     .unwrap(), // handle data validation results
    /// );
    /// assert_eq!(interp.interpolate(&[1.5]).unwrap(), 0.5);
    /// assert_eq!(interp.interpolate(&[-1.]).unwrap(), 0.); // extrapolation below grid
    /// assert_eq!(interp.interpolate(&[2.2]).unwrap(), 0.64); // extrapolation above grid
    /// ```
    Interp1D(Interp1D),
    /// Interpolates in two dimensions.
    /// Contains an [`Interp2D`] instance defining x, y, and f(x, y).
    ///
    /// Applicable interpolation strategies:
    /// - [`Strategy::Linear`]
    ///
    /// # Example (using [`Extrapolate::Clamp`]):
    /// ```
    /// use ninterp::*;
    /// let interp = Interpolator::Interp2D(
    ///     // f(x, y) = 0.2 * x + 0.4 * y
    ///     Interp2D::new(
    ///         vec![0., 1., 2.], // x0, x1, x2
    ///         vec![0., 1., 2.], // y0, y1, y2
    ///         vec![
    ///             vec![0.0, 0.4, 0.8], // f(x0, y0), f(x0, y1), f(x0, y2)
    ///             vec![0.2, 0.6, 1.0], // f(x1, y0), f(x1, y1), f(x1, y2)
    ///             vec![0.4, 0.8, 1.2], // f(x2, y0), f(x2, y1), f(x2, y2)
    ///         ],
    ///         Strategy::Linear,
    ///         Extrapolate::Clamp, // restrict point within grid bounds
    ///     )
    ///     .unwrap(),
    /// );
    /// assert_eq!(interp.interpolate(&[1.5, 1.5]).unwrap(), 0.9);
    /// assert_eq!(
    ///     interp.interpolate(&[-1., 2.5]).unwrap(),
    ///     interp.interpolate(&[0., 2.]).unwrap()
    /// ); // point is restricted to within grid bounds
    /// ```
    Interp2D(Interp2D),
    /// Interpolates in three dimensions.
    /// Contains an [`Interp3D`] instance defining x, y, z, and f(x, y, z).
    ///
    /// Applicable interpolation strategies:
    /// - [`Strategy::Linear`]
    ///
    /// # Example (using [`Extrapolate::Error`]):
    /// ```
    /// use ninterp::*;
    /// let interp = Interpolator::Interp3D(
    ///     // f(x, y, z) = 0.2 * x + 0.2 * y + 0.2 * z
    ///     Interp3D::new(
    ///         vec![1., 2.], // x0, x1
    ///         vec![1., 2.], // y0, y1
    ///         vec![1., 2.], // z0, z1
    ///         vec![
    ///             vec![
    ///                 vec![0.6, 0.8], // f(x0, y0, z0), f(x0, y0, z1)
    ///                 vec![0.8, 1.0], // f(x0, y1, z0), f(x0, y1, z1)
    ///             ],
    ///             vec![
    ///                 vec![0.8, 1.0], // f(x1, y0, z0), f(x1, y0, z1)
    ///                 vec![1.0, 1.2], // f(x1, y1, z0), f(x1, y1, z1)
    ///             ],
    ///         ],
    ///         Strategy::Linear,
    ///         Extrapolate::Error, // return an error when point is out of bounds
    ///     )
    ///     .unwrap(),
    /// );
    /// assert_eq!(interp.interpolate(&[1.5, 1.5, 1.5]).unwrap(), 0.9);
    /// // out of bounds point with `Extrapolate::Error` fails
    /// assert!(matches!(
    ///     interp.interpolate(&[2.5, 2.5, 2.5]).unwrap_err(),
    ///     InterpolationError::ExtrapolationError(_)
    /// ));
    /// ```
    Interp3D(Interp3D),
    /// Interpolates with any dimensionality.
    /// Contains an [`InterpND`] instance defining `grid` coordinates and function `values`.
    ///
    /// Applicable interpolation strategies:
    /// - [`Strategy::Linear`]
    ///
    /// # Example (using [`Extrapolate::Error`]):
    /// ```
    /// use ninterp::*;
    /// use ndarray::array;
    /// let interp = Interpolator::InterpND(
    ///     // f(x, y, z) = 0.2 * x + 0.2 * y + 0.2 * z
    ///     InterpND::new(
    ///         vec![
    ///             vec![1., 2.], // x0, x1
    ///             vec![1., 2.], // y0, y1
    ///             vec![1., 2.], // z0, z1
    ///         ], // grid coordinates
    ///         array![
    ///             [
    ///                 [0.6, 0.8], // f(x0, y0, z0), f(x0, y0, z1)
    ///                 [0.8, 1.0], // f(x0, y1, z0), f(x0, y1, z1)
    ///             ],
    ///             [
    ///                 [0.8, 1.0], // f(x1, y0, z0), f(x1, y0, z1)
    ///                 [1.0, 1.2], // f(x1, y1, z0), f(x1, y1, z1)
    ///             ],
    ///         ].into_dyn(), // values
    ///         Strategy::Linear,
    ///         Extrapolate::Error, // return an error when point is out of bounds
    ///     )
    ///     .unwrap(),
    /// );
    /// assert_eq!(interp.interpolate(&[1.5, 1.5, 1.5]).unwrap(), 0.9);
    /// // out of bounds point with `Extrapolate::Error` fails
    /// assert!(matches!(
    ///     interp.interpolate(&[2.5, 2.5, 2.5]).unwrap_err(),
    ///     InterpolationError::ExtrapolationError(_)
    /// ));
    /// ```
    InterpND(InterpND),
}

impl Interpolator {
    /// Interpolate at supplied point, after checking point validity.
    /// Length of supplied point must match interpolator dimensionality.
    pub fn interpolate(&self, point: &[f64]) -> Result<f64, InterpolationError> {
        self.validate_point(point)?;
        match self {
            Self::Interp0D(value) => Ok(*value),
            Self::Interp1D(interp) => {
                match interp.extrapolate {
                    Extrapolate::Clamp => {
                        let clamped_point =
                            &[point[0].clamp(interp.x[0], *interp.x.last().unwrap())];
                        return interp.interpolate(clamped_point);
                    }
                    Extrapolate::Error => {
                        if !(interp.x[0] <= point[0] && &point[0] <= interp.x.last().unwrap()) {
                            return Err(InterpolationError::ExtrapolationError(format!(
                                "point = {point:?}, grid = {:?}",
                                interp.x
                            )));
                        }
                    }
                    _ => {}
                };
                interp.interpolate(point)
            }
            Self::Interp2D(interp) => {
                match interp.extrapolate {
                    Extrapolate::Clamp => {
                        let clamped_point = &[
                            point[0].clamp(interp.x[0], *interp.x.last().unwrap()),
                            point[1].clamp(interp.y[0], *interp.y.last().unwrap()),
                        ];
                        return interp.interpolate(clamped_point);
                    }
                    Extrapolate::Error => {
                        if !(interp.x[0] <= point[0] && &point[0] <= interp.x.last().unwrap()) {
                            return Err(InterpolationError::ExtrapolationError(format!(
                                "point = {point:?}, x grid = {:?}",
                                interp.x
                            )));
                        }
                        if !(interp.y[0] <= point[1] && &point[1] <= interp.y.last().unwrap()) {
                            return Err(InterpolationError::ExtrapolationError(format!(
                                "point = {point:?}, y grid = {:?}",
                                interp.y
                            )));
                        }
                    }
                    _ => {}
                };
                interp.interpolate(point)
            }
            Self::Interp3D(interp) => {
                match interp.extrapolate {
                    Extrapolate::Clamp => {
                        let clamped_point = &[
                            point[0].clamp(interp.x[0], *interp.x.last().unwrap()),
                            point[1].clamp(interp.y[0], *interp.y.last().unwrap()),
                            point[2].clamp(interp.z[0], *interp.z.last().unwrap()),
                        ];
                        return interp.interpolate(clamped_point);
                    }
                    Extrapolate::Error => {
                        if !(interp.x[0] <= point[0] && &point[0] <= interp.x.last().unwrap()) {
                            return Err(InterpolationError::ExtrapolationError(format!(
                                "point = {point:?}, x grid = {:?}",
                                interp.x
                            )));
                        }
                        if !(interp.y[0] <= point[1] && &point[1] <= interp.y.last().unwrap()) {
                            return Err(InterpolationError::ExtrapolationError(format!(
                                "point = {point:?}, y grid = {:?}",
                                interp.y
                            )));
                        }
                        if !(interp.z[0] <= point[2] && &point[2] <= interp.z.last().unwrap()) {
                            return Err(InterpolationError::ExtrapolationError(format!(
                                "point = {point:?}, z grid = {:?}",
                                interp.z
                            )));
                        }
                    }
                    _ => {}
                };
                interp.interpolate(point)
            }
            Self::InterpND(interp) => {
                match interp.extrapolate {
                    Extrapolate::Clamp => {
                        let clamped_point: Vec<f64> = point
                            .iter()
                            .enumerate()
                            .map(|(dim, pt)| {
                                pt.clamp(interp.grid[dim][0], *interp.grid[dim].last().unwrap())
                            })
                            .collect();
                        return interp.interpolate(&clamped_point);
                    }
                    Extrapolate::Error => {
                        if !point.iter().enumerate().all(|(dim, pt_dim)| {
                            &interp.grid[dim][0] <= pt_dim
                                && pt_dim <= interp.grid[dim].last().unwrap()
                        }) {
                            return Err(InterpolationError::ExtrapolationError(format!(
                                "point = {point:?}, grid: {:?}",
                                interp.grid,
                            )));
                        }
                    }
                    _ => {}
                };
                interp.interpolate(point)
            }
        }
    }

    /// Ensure supplied point is valid for the given interpolator
    fn validate_point(&self, point: &[f64]) -> Result<(), InterpolationError> {
        let n = self.ndim();
        // Check supplied point dimensionality
        if n == 0 && !point.is_empty() {
            return Err(InterpolationError::InvalidPoint(
                "No point should be provided for 0-D interpolation".into(),
            ));
        } else if point.len() != n {
            return Err(InterpolationError::InvalidPoint(format!(
                "Supplied point slice should have length {n} for {n}-D interpolation"
            )));
        }
        Ok(())
    }

    /// Interpolator dimensionality
    fn ndim(&self) -> usize {
        match self {
            Self::Interp0D(_) => 0,
            Self::Interp1D(_) => 1,
            Self::Interp2D(_) => 2,
            Self::Interp3D(_) => 3,
            Self::InterpND(interp) => interp.ndim(),
        }
    }
}

// Getters and setters
impl Interpolator {
    /// Get `strategy` field
    pub fn strategy(&self) -> Result<&Strategy, Error> {
        match self {
            Interpolator::Interp1D(interp) => Ok(interp.strategy()),
            Interpolator::Interp2D(interp) => Ok(interp.strategy()),
            Interpolator::Interp3D(interp) => Ok(interp.strategy()),
            Interpolator::InterpND(interp) => Ok(interp.strategy()),
            _ => Err(Error::NoSuchField),
        }
    }

    /// Set `strategy` field
    pub fn set_strategy(&mut self, strategy: Strategy) -> Result<(), Error> {
        match self {
            Interpolator::Interp1D(interp) => Ok(interp.set_strategy(strategy)?),
            Interpolator::Interp2D(interp) => Ok(interp.set_strategy(strategy)?),
            Interpolator::Interp3D(interp) => Ok(interp.set_strategy(strategy)?),
            Interpolator::InterpND(interp) => Ok(interp.set_strategy(strategy)?),
            _ => return Err(Error::NoSuchField),
        }
    }

    /// Get `extrapolate` field
    pub fn extrapolate(&self) -> Result<&Extrapolate, Error> {
        match self {
            Interpolator::Interp1D(interp) => Ok(interp.extrapolate()),
            Interpolator::Interp2D(interp) => Ok(interp.extrapolate()),
            Interpolator::Interp3D(interp) => Ok(interp.extrapolate()),
            Interpolator::InterpND(interp) => Ok(interp.extrapolate()),
            _ => Err(Error::NoSuchField),
        }
    }

    /// Set `extrapolate` field
    pub fn set_extrapolate(&mut self, extrapolate: Extrapolate) -> Result<(), Error> {
        match self {
            Interpolator::Interp1D(interp) => Ok(interp.set_extrapolate(extrapolate)?),
            Interpolator::Interp2D(interp) => Ok(interp.set_extrapolate(extrapolate)?),
            Interpolator::Interp3D(interp) => Ok(interp.set_extrapolate(extrapolate)?),
            Interpolator::InterpND(interp) => Ok(interp.set_extrapolate(extrapolate)?),
            _ => return Err(Error::NoSuchField),
        }
    }

    /// Get `x` field
    pub fn x(&self) -> Result<&[f64], Error> {
        match self {
            Interpolator::Interp1D(interp) => Ok(interp.x()),
            Interpolator::Interp2D(interp) => Ok(interp.x()),
            Interpolator::Interp3D(interp) => Ok(interp.x()),
            _ => Err(Error::NoSuchField),
        }
    }

    /// Set `x` field
    pub fn set_x(&mut self, x: Vec<f64>) -> Result<(), Error> {
        match self {
            Interpolator::Interp1D(interp) => Ok(interp.set_x(x)?),
            Interpolator::Interp2D(interp) => Ok(interp.set_x(x)?),
            Interpolator::Interp3D(interp) => Ok(interp.set_x(x)?),
            _ => Err(Error::NoSuchField),
        }
    }

    /// Get `y` field
    pub fn y(&self) -> Result<&[f64], Error> {
        match self {
            Interpolator::Interp2D(interp) => Ok(interp.y()),
            Interpolator::Interp3D(interp) => Ok(interp.y()),
            _ => Err(Error::NoSuchField),
        }
    }

    /// Set `y` field
    pub fn set_y(&mut self, new_y: Vec<f64>) -> Result<(), Error> {
        match self {
            Interpolator::Interp2D(interp) => interp.set_y(new_y)?,
            Interpolator::Interp3D(interp) => interp.set_y(new_y)?,
            _ => return Err(Error::NoSuchField),
        }
        Ok(())
    }

    /// Get `z` field
    pub fn z(&self) -> Result<&[f64], Error> {
        match self {
            Interpolator::Interp3D(interp) => Ok(interp.z()),
            _ => Err(Error::NoSuchField),
        }
    }

    /// Set `z` field
    pub fn set_z(&mut self, new_z: Vec<f64>) -> Result<(), Error> {
        match self {
            Interpolator::Interp3D(interp) => Ok(interp.set_z(new_z)?),
            _ => Err(Error::NoSuchField),
        }
    }

    /// Get `f_x` field
    pub fn f_x(&self) -> Result<&[f64], Error> {
        match self {
            Interpolator::Interp1D(interp) => Ok(interp.f_x()),
            _ => Err(Error::NoSuchField),
        }
    }

    /// Set `f_x` field
    pub fn set_f_x(&mut self, f_x: Vec<f64>) -> Result<(), Error> {
        match self {
            Interpolator::Interp1D(interp) => Ok(interp.set_f_x(f_x)?),
            _ => Err(Error::NoSuchField),
        }
    }

    /// Get `f_xy` field
    pub fn f_xy(&self) -> Result<&[Vec<f64>], Error> {
        match self {
            Interpolator::Interp2D(interp) => Ok(interp.f_xy()),
            _ => Err(Error::NoSuchField),
        }
    }

    /// Set `f_xy` field
    pub fn set_f_xy(&mut self, new_f_xy: Vec<Vec<f64>>) -> Result<(), Error> {
        match self {
            Interpolator::Interp2D(interp) => Ok(interp.set_f_xy(new_f_xy)?),
            _ => Err(Error::NoSuchField),
        }
    }

    /// Get `f_xyz` field
    pub fn f_xyz(&self) -> Result<&[Vec<Vec<f64>>], Error> {
        match self {
            Interpolator::Interp3D(interp) => Ok(interp.f_xyz()),
            _ => Err(Error::NoSuchField),
        }
    }

    /// Set `f_xyz` field
    pub fn set_f_xyz(&mut self, new_f_xyz: Vec<Vec<Vec<f64>>>) -> Result<(), Error> {
        match self {
            Interpolator::Interp3D(interp) => Ok(interp.set_f_xyz(new_f_xyz)?),
            _ => Err(Error::NoSuchField),
        }
    }

    /// Get `grid` field
    pub fn grid(&self) -> Result<&[Vec<f64>], Error> {
        match self {
            Interpolator::InterpND(interp) => Ok(interp.grid()),
            _ => Err(Error::NoSuchField),
        }
    }

    /// Set `grid` field
    pub fn set_grid(&mut self, new_grid: Vec<Vec<f64>>) -> Result<(), Error> {
        match self {
            Interpolator::InterpND(interp) => Ok(interp.set_grid(new_grid)?),
            _ => Err(Error::NoSuchField),
        }
    }

    /// Get `values` field
    pub fn values(&self) -> Result<&ndarray::ArrayD<f64>, Error> {
        match self {
            Interpolator::InterpND(interp) => Ok(interp.values()),
            _ => Err(Error::NoSuchField),
        }
    }

    /// Set `values` gield
    pub fn set_values(&mut self, new_values: ndarray::ArrayD<f64>) -> Result<(), Error> {
        match self {
            Interpolator::InterpND(interp) => Ok(interp.set_values(new_values)?),
            _ => Err(Error::NoSuchField),
        }
    }
}

/// Interpolation strategy
#[derive(Clone, Debug, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum Strategy {
    /// Linear interpolation: <https://en.wikipedia.org/wiki/Linear_interpolation>
    #[default]
    Linear,
    /// Left-nearest (previous value) interpolation: <https://en.wikipedia.org/wiki/Nearest-neighbor_interpolation>
    LeftNearest,
    /// Right-nearest (next value) interpolation: <https://en.wikipedia.org/wiki/Nearest-neighbor_interpolation>
    RightNearest,
    /// Nearest value (left or right, whichever nearest) interpolation: <https://en.wikipedia.org/wiki/Nearest-neighbor_interpolation>
    Nearest,
}

/// Extrapolation strategy
///
/// Controls what happens if supplied interpolant point
/// is outside the bounds of the interpolation grid.
#[derive(Clone, Debug, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum Extrapolate {
    /// If interpolant point is beyond the limits of the interpolation grid,
    /// find result via extrapolation using slope of nearby points.  
    /// Currently only implemented for 1-D linear interpolation.
    Enable,
    /// Restrict interpolant point to the limits of the interpolation grid, using [`f64::clamp`].
    Clamp,
    /// Return an error when interpolant point is beyond the limits of the interpolation grid.
    #[default]
    Error,
}

/// Methods applicable to all interpolator helper structs
pub trait InterpMethods {
    /// Validate data stored in [Self]. By design, [Self] can be instantiatated
    /// only via the `new` method, which calls `validate`.
    fn validate(&self) -> Result<(), ValidationError>;
    /// Interpolate at given point
    fn interpolate(&self, point: &[f64]) -> Result<f64, InterpolationError>;
}

/// Linear interpolation: <https://en.wikipedia.org/wiki/Linear_interpolation>
pub trait Linear {
    fn linear(&self, point: &[f64]) -> Result<f64, InterpolationError>;
}

/// Left-nearest (previous value) interpolation: <https://en.wikipedia.org/wiki/Nearest-neighbor_interpolation>
pub trait LeftNearest {
    fn left_nearest(&self, point: &[f64]) -> Result<f64, InterpolationError>;
}

/// Right-nearest (next value) interpolation: <https://en.wikipedia.org/wiki/Nearest-neighbor_interpolation>
pub trait RightNearest {
    fn right_nearest(&self, point: &[f64]) -> Result<f64, InterpolationError>;
}

/// Nearest value (left or right, whichever nearest) interpolation: <https://en.wikipedia.org/wiki/Nearest-neighbor_interpolation>
pub trait Nearest {
    fn nearest(&self, point: &[f64]) -> Result<f64, InterpolationError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(non_snake_case)]
    fn test_0D() {
        let expected = 0.5;
        let interp = Interpolator::Interp0D(expected);
        assert_eq!(interp.interpolate(&[]).unwrap(), expected);
        assert!(matches!(
            interp.interpolate(&[0.]).unwrap_err(),
            InterpolationError::InvalidPoint(_)
        ));
    }
}
