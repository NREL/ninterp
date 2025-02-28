//! The `ninterp` crate provides
//! [multivariate interpolation](https://en.wikipedia.org/wiki/Multivariate_interpolation#Regular_grid)
//! over rectilinear grids of any dimensionality.
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
//! A prelude module has been defined: `use ninterp::prelude::*;`.
//! This exposes the types necessary for usage: [`Interpolator`], [`Strategy`], [`Extrapolate`], and the trait [`InterpMethods`].
//!
//! All interpolation is handled through instances of the [`Interpolator`] enum.
//!
//! Interpolation is executed by calling [`Interpolator::interpolate`].
//! The length of the supplied point slice must be equal to the interpolator dimensionality.
//! The interpolator dimensionality can be retrieved by calling [`Interpolator::ndim`].
//!
//! ## Note
//! For interpolators of dimensionality N ≥ 1:
//! - Instantiation is done via the Interpolator enum's `new_*` methods (`new_1d`, `new_2d`, `new_3d`, `new_nd`).
//! These methods run a validation step that catches any potential errors early, preventing runtime panics.
//!   - To set or get field values, use the corresponding named methods (`x`, `set_x`, etc.).
//! - An interpolation [`Strategy`] (e.g. linear, left-nearest, etc.) must be specified.
//! Not all interpolation strategies are implemented for every dimensionality.
//! [`Strategy::Linear`] is implemented for all dimensionalities.
//! - An [`Extrapolate`] setting must be specified.
//! This controls what happens when a point is beyond the range of supplied coordinates.
//! If you are unsure which variant to choose, [`Extrapolate::Error`] is likely what you want.
//!
//! For 0-D (constant-value) interpolators, instantiate directly, e.g. `Interpolator::Interp0D(0.5)`
//!
//! ## Examples
//! - [`Interpolator::Interp0D`]
//! - [`Interpolator::new_1d`]
//! - [`Interpolator::new_2d`]
//! - [`Interpolator::new_3d`]
//! - [`Interpolator::new_nd`]
//!

pub mod prelude {
    pub use crate::{Extrapolate, InterpMethods, Interpolator, Strategy};
}

pub mod error;
mod n;
mod one;
mod three;
mod traits;
mod two;

pub(crate) use error::*;
pub(crate) use n::*;
pub(crate) use one::*;
pub(crate) use three::*;
pub use traits::*;
pub(crate) use two::*;

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
#[allow(private_interfaces)]
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum Interpolator {
    /// Returns a constant value.
    ///
    /// # Example:
    /// ```
    /// use ninterp::prelude::*;
    /// // 0-D is unique, the value is directly provided in the variant
    /// let const_value = 0.5;
    /// let interp = Interpolator::Interp0D(const_value);
    /// assert_eq!(
    ///     interp.interpolate(&[]).unwrap(), // an empty point is required for 0-D
    ///     const_value
    /// );
    /// ```
    Interp0D(f64),
    /// Interpolates in one dimension.
    ///
    /// See [`Interpolator::new_1d`] documentation for example usage.
    Interp1D(Interp1D),
    /// Interpolates in two dimensions.
    ///
    /// See [`Interpolator::new_2d`] documentation for example usage.
    Interp2D(Interp2D),
    /// Interpolates in three dimensions.
    ///
    /// See [`Interpolator::new_3d`] documentation for example usage.
    Interp3D(Interp3D),
    /// Interpolates with any dimensionality.
    ///
    /// See [`Interpolator::new_nd`] documentation for example usage.
    InterpND(InterpND),
}

impl Interpolator {
    #[deprecated(note = "instantiate directly via `Interpolator::Interp0D(value)` instead")]
    pub fn new_0d(value: f64) -> Result<Self, ValidationError> {
        Ok(Self::Interp0D(value))
    }

    /// Instantiate one-dimensional interpolator.
    ///
    /// Applicable interpolation strategies:
    /// - [`Strategy::Linear`]
    /// - [`Strategy::LeftNearest`]
    /// - [`Strategy::RightNearest`]
    /// - [`Strategy::Nearest`]
    ///
    /// Applicable extrapolation strategies:
    /// - [`Extrapolate::Clamp`]
    /// - [`Extrapolate::Error`]
    ///
    /// # Example (using [`Extrapolate::Clamp`]):
    /// ```
    /// use ninterp::prelude::*;
    /// // f(x, y) = 0.2 * x + 0.4 * y
    /// let interp = Interpolator::new_2d(
    ///     // x
    ///     vec![0., 1., 2.], // x0, x1, x2
    ///     // y
    ///     vec![0., 1., 2.], // y0, y1, y2
    ///     // f(x, y)
    ///     vec![
    ///         vec![0.0, 0.4, 0.8], // f(x0, y0), f(x0, y1), f(x0, y2)
    ///         vec![0.2, 0.6, 1.0], // f(x1, y0), f(x1, y1), f(x1, y2)
    ///         vec![0.4, 0.8, 1.2], // f(x2, y0), f(x2, y1), f(x2, y2)
    ///     ],
    ///     Strategy::Linear,
    ///     Extrapolate::Clamp, // restrict point within grid bounds
    /// )
    /// .unwrap();
    /// assert_eq!(interp.interpolate(&[1.5, 1.5]).unwrap(), 0.9);
    /// assert_eq!(
    ///     interp.interpolate(&[-1., 2.5]).unwrap(),
    ///     interp.interpolate(&[0., 2.]).unwrap()
    /// ); // point is restricted to within grid bounds
    /// ```
    pub fn new_1d(
        x: Vec<f64>,
        f_x: Vec<f64>,
        strategy: Strategy,
        extrapolate: Extrapolate,
    ) -> Result<Self, ValidationError> {
        let interp = Interp1D {
            x,
            f_x,
            strategy,
            extrapolate,
        };
        interp.validate()?;
        Ok(Self::Interp1D(interp))
    }

    /// Instantiate two-dimensional interpolator.
    ///
    /// Applicable interpolation strategies:
    /// - [`Strategy::Linear`]
    /// - [`Strategy::Nearest`]
    ///
    /// Applicable extrapolation strategies:
    /// - [`Extrapolate::Clamp`]
    /// - [`Extrapolate::Error`]
    ///
    /// # Example (using [`Extrapolate::Clamp`]):
    /// ```
    /// use ninterp::prelude::*;
    /// // f(x, y) = 0.2 * x + 0.4 * y
    /// let interp = Interpolator::new_2d(
    ///     // x
    ///     vec![0., 1., 2.], // x0, x1, x2
    ///     // y
    ///     vec![0., 1., 2.], // y0, y1, y2
    ///     // f(x, y)
    ///     vec![
    ///         vec![0.0, 0.4, 0.8], // f(x0, y0), f(x0, y1), f(x0, y2)
    ///         vec![0.2, 0.6, 1.0], // f(x1, y0), f(x1, y1), f(x1, y2)
    ///         vec![0.4, 0.8, 1.2], // f(x2, y0), f(x2, y1), f(x2, y2)
    ///     ],
    ///     Strategy::Linear,
    ///     Extrapolate::Clamp, // restrict point within grid bounds
    /// )
    /// .unwrap();
    /// assert_eq!(interp.interpolate(&[1.5, 1.5]).unwrap(), 0.9);
    /// assert_eq!(
    ///     interp.interpolate(&[-1., 2.5]).unwrap(),
    ///     interp.interpolate(&[0., 2.]).unwrap()
    /// ); // point is restricted to within grid bounds
    /// ```
    pub fn new_2d(
        x: Vec<f64>,
        y: Vec<f64>,
        f_xy: Vec<Vec<f64>>,
        strategy: Strategy,
        extrapolate: Extrapolate,
    ) -> Result<Self, ValidationError> {
        let interp = Interp2D {
            x,
            y,
            f_xy,
            strategy,
            extrapolate,
        };
        interp.validate()?;
        Ok(Self::Interp2D(interp))
    }

    /// Instantiate three-dimensional interpolator.
    ///
    /// Applicable interpolation strategies:
    /// - [`Strategy::Linear`]
    /// - [`Strategy::Nearest`]
    ///
    /// Applicable extrapolation strategies:
    /// - [`Extrapolate::Clamp`]
    /// - [`Extrapolate::Error`]
    ///
    /// # Example (using [`Extrapolate::Error`]):
    /// ```
    /// use ninterp::prelude::*;
    /// // f(x, y, z) = 0.2 * x + 0.2 * y + 0.2 * z
    /// let interp = Interpolator::new_3d(
    ///     // x
    ///     vec![1., 2.], // x0, x1
    ///     // y
    ///     vec![1., 2.], // y0, y1
    ///     // z
    ///     vec![1., 2.], // z0, z1
    ///     // f(x, y, z)
    ///     vec![
    ///         vec![
    ///             vec![0.6, 0.8], // f(x0, y0, z0), f(x0, y0, z1)
    ///             vec![0.8, 1.0], // f(x0, y1, z0), f(x0, y1, z1)
    ///         ],
    ///         vec![
    ///             vec![0.8, 1.0], // f(x1, y0, z0), f(x1, y0, z1)
    ///             vec![1.0, 1.2], // f(x1, y1, z0), f(x1, y1, z1)
    ///         ],
    ///     ],
    ///     Strategy::Linear,
    ///     Extrapolate::Error, // return an error when point is out of bounds
    /// )
    /// .unwrap();
    /// assert_eq!(interp.interpolate(&[1.5, 1.5, 1.5]).unwrap(), 0.9);
    /// // out of bounds point with `Extrapolate::Error` fails
    /// assert!(matches!(
    ///     interp.interpolate(&[2.5, 2.5, 2.5]).unwrap_err(),
    ///     ninterp::error::InterpolationError::ExtrapolationError(_)
    /// ));
    /// ```
    pub fn new_3d(
        x: Vec<f64>,
        y: Vec<f64>,
        z: Vec<f64>,
        f_xyz: Vec<Vec<Vec<f64>>>,
        strategy: Strategy,
        extrapolate: Extrapolate,
    ) -> Result<Self, ValidationError> {
        let interp = Interp3D {
            x,
            y,
            z,
            f_xyz,
            strategy,
            extrapolate,
        };
        interp.validate()?;
        Ok(Self::Interp3D(interp))
    }

    /// Instantiate N-dimensional (any dimensionality) interpolator.
    ///
    /// Applicable interpolation strategies:
    /// - [`Strategy::Linear`]
    /// - [`Strategy::Nearest`]
    ///
    /// Applicable extrapolation strategies:
    /// - [`Extrapolate::Clamp`]
    /// - [`Extrapolate::Error`]
    ///
    /// # Example (using [`Extrapolate::Error`]):
    /// ```
    /// use ninterp::prelude::*;
    /// // f(x, y, z) = 0.2 * x + 0.2 * y + 0.2 * z
    /// let interp = Interpolator::new_nd(
    ///     // grid
    ///     vec![
    ///         vec![1., 2.], // x0, x1
    ///         vec![1., 2.], // y0, y1
    ///         vec![1., 2.], // z0, z1
    ///     ],
    ///     // values
    ///     ndarray::array![
    ///         [
    ///             [0.6, 0.8], // f(x0, y0, z0), f(x0, y0, z1)
    ///             [0.8, 1.0], // f(x0, y1, z0), f(x0, y1, z1)
    ///         ],
    ///         [
    ///             [0.8, 1.0], // f(x1, y0, z0), f(x1, y0, z1)
    ///             [1.0, 1.2], // f(x1, y1, z0), f(x1, y1, z1)
    ///         ],
    ///     ].into_dyn(),
    ///     Strategy::Linear,
    ///     Extrapolate::Error, // return an error when point is out of bounds
    /// )
    /// .unwrap();
    /// assert_eq!(interp.interpolate(&[1.5, 1.5, 1.5]).unwrap(), 0.9);
    /// // out of bounds point with `Extrapolate::Error` fails
    /// assert!(matches!(
    ///     interp.interpolate(&[2.5, 2.5, 2.5]).unwrap_err(),
    ///     ninterp::error::InterpolationError::ExtrapolationError(_)
    /// ));
    /// ```
    pub fn new_nd(
        grid: Vec<Vec<f64>>,
        values: ndarray::ArrayD<f64>,
        strategy: Strategy,
        extrapolate: Extrapolate,
    ) -> Result<Self, ValidationError> {
        let interp = InterpND {
            grid,
            values,
            strategy,
            extrapolate,
        };
        interp.validate()?;
        Ok(Self::InterpND(interp))
    }

    /// Ensure supplied point is valid for the given interpolator.
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

    /// Retrieve interpolator dimensionality.
    pub fn ndim(&self) -> usize {
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
    /// Get `strategy` field from any interpolator.
    pub fn strategy(&self) -> Result<&Strategy, Error> {
        match self {
            Self::Interp1D(interp) => Ok(&interp.strategy),
            Self::Interp2D(interp) => Ok(&interp.strategy),
            Self::Interp3D(interp) => Ok(&interp.strategy),
            Self::InterpND(interp) => Ok(&interp.strategy),
            _ => Err(Error::NoSuchField),
        }
    }

    /// Set `strategy` field on any interpolator.
    pub fn set_strategy(&mut self, strategy: Strategy) -> Result<(), Error> {
        match self {
            Self::Interp1D(interp) => {
                interp.strategy = strategy;
                Ok(interp.validate()?)
            }
            Self::Interp2D(interp) => {
                interp.strategy = strategy;
                Ok(interp.validate()?)
            }
            Self::Interp3D(interp) => {
                interp.strategy = strategy;
                Ok(interp.validate()?)
            }
            Self::InterpND(interp) => {
                interp.strategy = strategy;
                Ok(interp.validate()?)
            }
            _ => Err(Error::NoSuchField),
        }
    }

    /// Get `extrapolate` field from any interpolator.
    pub fn extrapolate(&self) -> Result<&Extrapolate, Error> {
        match self {
            Self::Interp1D(interp) => Ok(&interp.extrapolate),
            Self::Interp2D(interp) => Ok(&interp.extrapolate),
            Self::Interp3D(interp) => Ok(&interp.extrapolate),
            Self::InterpND(interp) => Ok(&interp.extrapolate),
            _ => Err(Error::NoSuchField),
        }
    }

    /// Set `extrapolate` field on any interpolator.
    pub fn set_extrapolate(&mut self, extrapolate: Extrapolate) -> Result<(), Error> {
        match self {
            Self::Interp1D(interp) => {
                interp.extrapolate = extrapolate;
                Ok(interp.validate()?)
            }
            Self::Interp2D(interp) => {
                interp.extrapolate = extrapolate;
                Ok(interp.validate()?)
            }
            Self::Interp3D(interp) => {
                interp.extrapolate = extrapolate;
                Ok(interp.validate()?)
            }
            Self::InterpND(interp) => {
                interp.extrapolate = extrapolate;
                Ok(interp.validate()?)
            }
            _ => Err(Error::NoSuchField),
        }
    }

    /// Get `x` field from 1D/2D/3D interpolator.
    pub fn x(&self) -> Result<&[f64], Error> {
        match self {
            Self::Interp1D(interp) => Ok(&interp.x),
            Self::Interp2D(interp) => Ok(&interp.x),
            Self::Interp3D(interp) => Ok(&interp.x),
            _ => Err(Error::NoSuchField),
        }
    }

    /// Set `x` field on 1D/2D/3D interpolator.
    pub fn set_x(&mut self, x: Vec<f64>) -> Result<(), Error> {
        match self {
            Self::Interp1D(interp) => {
                interp.x = x;
                Ok(interp.validate()?)
            }
            Self::Interp2D(interp) => {
                interp.x = x;
                Ok(interp.validate()?)
            }
            Self::Interp3D(interp) => {
                interp.x = x;
                Ok(interp.validate()?)
            }
            _ => Err(Error::NoSuchField),
        }
    }

    /// Get `y` field from 2D/3D interpolator.
    pub fn y(&self) -> Result<&[f64], Error> {
        match self {
            Self::Interp2D(interp) => Ok(&interp.y),
            Self::Interp3D(interp) => Ok(&interp.y),
            _ => Err(Error::NoSuchField),
        }
    }

    /// Set `y` field on 2D/3D interpolator.
    pub fn set_y(&mut self, y: Vec<f64>) -> Result<(), Error> {
        match self {
            Self::Interp2D(interp) => {
                interp.y = y;
                Ok(interp.validate()?)
            }
            Self::Interp3D(interp) => {
                interp.y = y;
                Ok(interp.validate()?)
            }
            _ => Err(Error::NoSuchField),
        }
    }

    /// Get `z` field from 3D interpolator.
    pub fn z(&self) -> Result<&[f64], Error> {
        match self {
            Self::Interp3D(interp) => Ok(&interp.z),
            _ => Err(Error::NoSuchField),
        }
    }

    /// Set `z` field on 3D interpolator.
    pub fn set_z(&mut self, z: Vec<f64>) -> Result<(), Error> {
        match self {
            Self::Interp3D(interp) => {
                interp.z = z;
                Ok(interp.validate()?)
            }
            _ => Err(Error::NoSuchField),
        }
    }

    /// Get `f_x` field from 1D interpolator.
    pub fn f_x(&self) -> Result<&[f64], Error> {
        match self {
            Self::Interp1D(interp) => Ok(&interp.f_x),
            _ => Err(Error::NoSuchField),
        }
    }

    /// Set `f_x` field on 1D interpolator.
    pub fn set_f_x(&mut self, f_x: Vec<f64>) -> Result<(), Error> {
        match self {
            Self::Interp1D(interp) => {
                interp.f_x = f_x;
                Ok(interp.validate()?)
            }
            _ => Err(Error::NoSuchField),
        }
    }

    /// Get `f_xy` field from 2D interpolator.
    pub fn f_xy(&self) -> Result<&[Vec<f64>], Error> {
        match self {
            Self::Interp2D(interp) => Ok(&interp.f_xy),
            _ => Err(Error::NoSuchField),
        }
    }

    /// Set `f_xy` field on 2D interpolator.
    pub fn set_f_xy(&mut self, f_xy: Vec<Vec<f64>>) -> Result<(), Error> {
        match self {
            Self::Interp2D(interp) => {
                interp.f_xy = f_xy;
                Ok(interp.validate()?)
            }
            _ => Err(Error::NoSuchField),
        }
    }

    /// Get `f_xyz` field from 3D interpolator.
    pub fn f_xyz(&self) -> Result<&[Vec<Vec<f64>>], Error> {
        match self {
            Self::Interp3D(interp) => Ok(&interp.f_xyz),
            _ => Err(Error::NoSuchField),
        }
    }

    /// Set `f_xyz` field on 3D interpolator.
    pub fn set_f_xyz(&mut self, f_xyz: Vec<Vec<Vec<f64>>>) -> Result<(), Error> {
        match self {
            Self::Interp3D(interp) => {
                interp.f_xyz = f_xyz;
                Ok(interp.validate()?)
            }
            _ => Err(Error::NoSuchField),
        }
    }

    /// Get `grid` field from ND interpolator.
    pub fn grid(&self) -> Result<&[Vec<f64>], Error> {
        match self {
            Self::InterpND(interp) => Ok(&interp.grid),
            _ => Err(Error::NoSuchField),
        }
    }

    /// Set `grid` field on ND interpolator.
    pub fn set_grid(&mut self, grid: Vec<Vec<f64>>) -> Result<(), Error> {
        match self {
            Self::InterpND(interp) => {
                interp.grid = grid;
                Ok(interp.validate()?)
            }
            _ => Err(Error::NoSuchField),
        }
    }

    /// Get `values` field from ND interpolator.
    pub fn values(&self) -> Result<&ndarray::ArrayD<f64>, Error> {
        match self {
            Self::InterpND(interp) => Ok(&interp.values),
            _ => Err(Error::NoSuchField),
        }
    }

    /// Set `values` field on ND interpolator.
    pub fn set_values(&mut self, values: ndarray::ArrayD<f64>) -> Result<(), Error> {
        match self {
            Self::InterpND(interp) => {
                interp.values = values;
                Ok(interp.validate()?)
            }
            _ => Err(Error::NoSuchField),
        }
    }
}

impl InterpMethods for Interpolator {
    fn validate(&self) -> Result<(), ValidationError> {
        match self {
            Self::Interp0D(_) => Ok(()),
            Self::Interp1D(interp) => interp.validate(),
            Self::Interp2D(interp) => interp.validate(),
            Self::Interp3D(interp) => interp.validate(),
            Self::InterpND(interp) => interp.validate(),
        }
    }

    /// Interpolate at supplied point, after checking point validity.
    /// Length of supplied point must match interpolator dimensionality.
    fn interpolate(&self, point: &[f64]) -> Result<f64, InterpolationError> {
        self.validate_point(point)?;
        match self {
            Self::Interp0D(value) => Ok(*value),
            Self::Interp1D(interp) => {
                match interp.extrapolate {
                    Extrapolate::Clamp => {
                        let clamped_point =
                            &[point[0]
                                .clamp(*interp.x.first().unwrap(), *interp.x.last().unwrap())];
                        return interp.interpolate(clamped_point);
                    }
                    Extrapolate::Error => {
                        if !(interp.x.first().unwrap()..=interp.x.last().unwrap())
                            .contains(&&point[0])
                        {
                            return Err(InterpolationError::ExtrapolationError(format!(
                                "\n    point[0] = {:?} is out of bounds for x-grid = {:?}",
                                point[0], interp.x
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
                            point[0].clamp(*interp.x.first().unwrap(), *interp.x.last().unwrap()),
                            point[1].clamp(*interp.y.first().unwrap(), *interp.y.last().unwrap()),
                        ];
                        return interp.interpolate(clamped_point);
                    }
                    Extrapolate::Error => {
                        let grid = [&interp.x, &interp.y];
                        let grid_names = ["x", "y"];
                        let mut errors = Vec::new();
                        for dim in 0..2 {
                            if !(grid[dim].first().unwrap()..=grid[dim].last().unwrap())
                                .contains(&&point[dim])
                            {
                                errors.push(format!(
                                    "\n    point[{dim}] = {:?} is out of bounds for {}-grid = {:?}",
                                    point[dim], grid_names[dim], grid[dim],
                                ));
                            }
                        }
                        if !errors.is_empty() {
                            return Err(InterpolationError::ExtrapolationError(errors.join("")));
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
                            point[0].clamp(*interp.x.first().unwrap(), *interp.x.last().unwrap()),
                            point[1].clamp(*interp.x.first().unwrap(), *interp.y.last().unwrap()),
                            point[2].clamp(*interp.x.first().unwrap(), *interp.z.last().unwrap()),
                        ];
                        return interp.interpolate(clamped_point);
                    }
                    Extrapolate::Error => {
                        let grid = [&interp.x, &interp.y, &interp.z];
                        let grid_names = ["x", "y", "z"];
                        let mut errors = Vec::new();
                        for dim in 0..3 {
                            if !(grid[dim].first().unwrap()..=grid[dim].last().unwrap())
                                .contains(&&point[dim])
                            {
                                errors.push(format!(
                                    "\n    point[{dim}] = {:?} is out of bounds for {}-grid = {:?}",
                                    point[dim], grid_names[dim], grid[dim],
                                ));
                            }
                        }
                        if !errors.is_empty() {
                            return Err(InterpolationError::ExtrapolationError(errors.join("")));
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
                                pt.clamp(
                                    *interp.grid[dim].first().unwrap(),
                                    *interp.grid[dim].last().unwrap(),
                                )
                            })
                            .collect();
                        return interp.interpolate(&clamped_point);
                    }
                    Extrapolate::Error => {
                        let mut errors = Vec::new();
                        for dim in 0..interp.ndim() {
                            if !(interp.grid[dim].first().unwrap()
                                ..=interp.grid[dim].last().unwrap())
                                .contains(&&point[dim])
                            {
                                errors.push(format!(
                                    "\n    point[{dim}] = {:?} is out of bounds for grid[{dim}] = {:?}",
                                    point[dim],
                                    interp.grid[dim],
                                ));
                            }
                        }
                        if !errors.is_empty() {
                            return Err(InterpolationError::ExtrapolationError(errors.join("")));
                        }
                    }
                    _ => {}
                };
                interp.interpolate(point)
            }
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
    /// Nearest value interpolation: <https://en.wikipedia.org/wiki/Nearest-neighbor_interpolation>
    ///
    /// # Note
    /// Float imprecision may affect the value returned near midpoints.
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
