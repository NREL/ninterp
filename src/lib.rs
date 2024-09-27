pub mod error;
pub mod n;
pub mod one;
pub mod three;
pub mod two;

pub use error::*;
pub use one::*;
pub use three::*;
pub use two::*;

#[cfg(feature = "nd")]
pub use n::*;

use anyhow::Context;

// This method contains code from RouteE Compass, another NREL-developed tool
// https://www.nrel.gov/transportation/route-energy-prediction-model.html
// https://github.com/NREL/routee-compass/
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

/// # 0-D (constant value) example:
/// ```
/// use ninterp::*;
/// // 0-D is unique, the value is directly provided in the variant
/// let const_value = 0.5;
/// let interp = Interpolator::Interp0D(const_value);
/// assert_eq!(interp.interpolate(&[]).unwrap(), const_value); // an empty point is required for 0-D
/// ```
///
/// # 1-D example (linear, with extrapolation):
/// ```
/// use ninterp::*;
/// let interp = Interpolator::Interp1D(
///     // f(x) = 0.2 * x + 0.2
///     Interp1D::new(
///         vec![0., 1., 2.], // x0, x1, x2
///         vec![0.2, 0.4, 0.6], // f(x0), f(x1), f(x2)
///         Strategy::Linear, // linear interpolation
///         Extrapolate::Extrapolate, // linearly extrapolate when point is out of bounds
///     )
///     .unwrap(), // handle data validation results
/// );
/// assert_eq!(interp.interpolate(&[1.5]).unwrap(), 0.5);
/// assert_eq!(interp.interpolate(&[-1.]).unwrap(), 0.); // extrapolation below grid
/// assert_eq!(interp.interpolate(&[2.2]).unwrap(), 0.64); // extrapolation above grid
/// ```
///
/// # 2-D example (linear, using [`Extrapolate::Clamp`]):
/// ```
/// use ninterp::*;
/// let interp = Interpolator::Interp2D(
///     // f(x) = 0.2 * x + 0.4 * y
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
///
/// # 3-D example (linear, using [`Extrapolate::Error`]):
/// ```
/// use ninterp::*;
/// let interp = Interpolator::Interp3D(
///     // f(x) = 0.2 * x + 0.2 * y + 0.2 * z
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
/// assert!(interp.interpolate(&[2.5, 2.5, 2.5]).is_err()); // out of bounds point with `Extrapolate::Error` fails
/// ```
///
/// # N-D example (same as 3-D):
/// ```
/// #[cfg(feature = "nd")] {
///    use ninterp::*;
///    use ndarray::array;
///     let interp = Interpolator::InterpND(
///         // f(x) = 0.2 * x + 0.2 * y + 0.2 * z
///         InterpND::new(
///             vec![
///                 vec![1., 2.], // x0, x1
///                 vec![1., 2.], // y0, y1
///                 vec![1., 2.], // z0, z1
///             ], // grid coordinates
///             array![
///                 [
///                     [0.6, 0.8], // f(x0, y0, z0), f(x0, y0, z1)
///                     [0.8, 1.0], // f(x0, y1, z0), f(x0, y1, z1)
///                 ],
///                 [
///                     [0.8, 1.0], // f(x1, y0, z0), f(x1, y0, z1)
///                     [1.0, 1.2], // f(x1, y1, z0), f(x1, y1, z1)
///                 ],
///             ].into_dyn(), // values
///             Strategy::Linear,
///             Extrapolate::Error, // return an error when point is out of bounds
///         )
///         .unwrap(),
///     );
///     assert_eq!(interp.interpolate(&[1.5, 1.5, 1.5]).unwrap(), 0.9);
///     assert!(interp.interpolate(&[2.5, 2.5, 2.5]).is_err()); // out of bounds point with `Extrapolate::Error` fails
/// }
/// ```
///
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", Deserialize, Serialize)]
pub enum Interpolator {
    /// 0-dimensional (constant value) interpolation
    Interp0D(f64),
    /// 1-dimensional interpolation
    Interp1D(Interp1D),
    /// 2-dimensional interpolation
    Interp2D(Interp2D),
    /// 3-dimensional interpolation
    Interp3D(Interp3D),
    /// N-dimensional interpolation
    #[cfg(feature = "nd")]
    InterpND(InterpND),
}

impl Interpolator {
    /// Interpolate at supplied point, after checking point validity.
    /// Length of supplied point must match interpolator dimensionality.
    pub fn interpolate(&self, point: &[f64]) -> anyhow::Result<f64> {
        self.validate_inputs(point)?;
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
                        let x_dim_ok =
                            interp.x[0] <= point[0] && &point[0] <= interp.x.last().unwrap();
                        anyhow::ensure!(
                            x_dim_ok,
                            "Attempted to interpolate at point beyond grid data: point = {point:?}, grid = {:?}",
                            interp.x,
                        );
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
                        let x_dim_ok =
                            interp.x[0] <= point[0] && &point[0] <= interp.x.last().unwrap();
                        let y_dim_ok =
                            interp.y[0] <= point[1] && &point[1] <= interp.y.last().unwrap();
                        anyhow::ensure!(
                            x_dim_ok && y_dim_ok,
                            "Attempted to interpolate at point beyond grid data: point = {point:?}, x grid = {:?}, y grid = {:?}",
                            interp.x,
                            interp.y,
                        );
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
                        let x_dim_ok =
                            interp.x[0] <= point[0] && &point[0] <= interp.x.last().unwrap();
                        let y_dim_ok =
                            interp.y[0] <= point[1] && &point[1] <= interp.y.last().unwrap();
                        let z_dim_ok =
                            interp.z[0] <= point[2] && &point[2] <= interp.z.last().unwrap();
                        anyhow::ensure!(x_dim_ok && y_dim_ok && z_dim_ok,
                            "Attempted to interpolate at point beyond grid data: point = {point:?}, x grid = {:?}, y grid = {:?}, z grid = {:?}",
                            interp.x,
                            interp.y,
                            interp.z,
                        );
                    }
                    _ => {}
                };
                interp.interpolate(point)
            }
            #[cfg(feature = "nd")]
            Self::InterpND(interp) => {
                match interp.extrapolate {
                    Extrapolate::Clamp => {
                        let clamped_point: Vec<f64> = point
                            .iter()
                            .enumerate()
                            .map(|(dim, pt)|
                                pt.clamp(interp.grid[dim][0], *interp.grid[dim].last().unwrap())
                            ).collect();
                        return interp.interpolate(&clamped_point);
                    }
                    Extrapolate::Error => anyhow::ensure!(
                        point.iter().enumerate().all(|(dim, pt_dim)| &interp.grid[dim][0] <= pt_dim && pt_dim <= interp.grid[dim].last().unwrap()),
                        "Attempted to interpolate at point beyond grid data: point = {point:?}, grid: {:?}",
                        interp.grid,
                    ),
                    _ => {}
                };
                interp.interpolate(point)
            }
        }
    }

    /// Ensure that point is valid for the interpolator instance.
    fn validate_inputs(&self, point: &[f64]) -> anyhow::Result<()> {
        let n = self.ndim();
        // Check supplied point dimensionality
        if n == 0 {
            anyhow::ensure!(
                point.is_empty(),
                "No point should be provided for 0-D interpolation"
            )
        } else {
            anyhow::ensure!(
                point.len() == n,
                "Supplied point slice should have length {n} for {n}-D interpolation"
            )
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
            #[cfg(feature = "nd")]
            Self::InterpND(interp) => interp.ndim(),
        }
    }

    /// Function to get x variable from enum variants
    pub fn x(&self) -> anyhow::Result<Vec<f64>> {
        match self {
            Interpolator::Interp1D(interp) => Ok(interp.x.to_vec()),
            Interpolator::Interp2D(interp) => Ok(interp.x.to_vec()),
            Interpolator::Interp3D(interp) => Ok(interp.x.to_vec()),
            _ => anyhow::bail!("Variant does not have `x` field."),
        }
    }

    /// Function to set x variable from enum variants
    /// # Arguments
    /// - `new_x`: updated `x` variable to replace the current `x` variable
    pub fn set_x(&mut self, new_x: Vec<f64>) -> anyhow::Result<()> {
        match self {
            Interpolator::Interp1D(interp) => interp.set_x(new_x)?,
            Interpolator::Interp2D(interp) => interp.set_x(new_x)?,
            Interpolator::Interp3D(interp) => interp.set_x(new_x)?,
            #[cfg(feature = "nd")]
            Interpolator::InterpND(interp) => interp.set_grid_x(new_x)?,
            _ => {
                anyhow::bail!(
                    "Variant does not have `x` field (or a grid row 0 in the case of InterpND)."
                )
            }
        }
        Ok(())
    }

    /// Function to get f_x variable from enum variants
    pub fn f_x(&self) -> anyhow::Result<Vec<f64>> {
        match self {
            Interpolator::Interp1D(interp) => Ok(interp.f_x.to_vec()),
            _ => anyhow::bail!("Variant does not have `f_x` field."),
        }
    }

    /// Function to set f_x variable from enum variants
    /// # Arguments
    /// - `new_f_x`: updated `f_x` variable to replace the current `f_x` variable
    pub fn set_f_x(&mut self, new_f_x: Vec<f64>) -> anyhow::Result<()> {
        match self {
            Interpolator::Interp1D(interp) => interp.set_f_x(new_f_x)?,
            _ => anyhow::bail!("Variant does not have `f_x` field."),
        }
        Ok(())
    }

    /// Function to get strategy variable from enum variants
    pub fn strategy(&self) -> anyhow::Result<Strategy> {
        match self {
            Interpolator::Interp1D(interp) => Ok(interp.strategy.to_owned()),
            Interpolator::Interp2D(interp) => Ok(interp.strategy.to_owned()),
            Interpolator::Interp3D(interp) => Ok(interp.strategy.to_owned()),
            #[cfg(feature = "nd")]
            Interpolator::InterpND(interp) => Ok(interp.strategy.to_owned()),
            _ => anyhow::bail!("Variant does not have `strategy` field."),
        }
    }

    /// Function to set strategy variable from enum variants
    /// # Arguments
    /// - `new_strategy`: updated `strategy` variable to replace the current `strategy` variable
    pub fn set_strategy(&mut self, new_strategy: Strategy) -> anyhow::Result<()> {
        match self {
            Interpolator::Interp1D(interp) => interp.strategy = new_strategy,
            Interpolator::Interp2D(interp) => interp.strategy = new_strategy,
            Interpolator::Interp3D(interp) => interp.strategy = new_strategy,
            #[cfg(feature = "nd")]
            Interpolator::InterpND(interp) => interp.strategy = new_strategy,
            _ => anyhow::bail!("Variant does not have `strategy` field."),
        }
        Ok(())
    }

    /// Function to get extrapolate variable from enum variants
    pub fn extrapolate(&self) -> anyhow::Result<Extrapolate> {
        match self {
            Interpolator::Interp1D(interp) => Ok(interp.extrapolate.to_owned()),
            Interpolator::Interp2D(interp) => Ok(interp.extrapolate.to_owned()),
            Interpolator::Interp3D(interp) => Ok(interp.extrapolate.to_owned()),
            #[cfg(feature = "nd")]
            Interpolator::InterpND(interp) => Ok(interp.extrapolate.to_owned()),
            _ => anyhow::bail!("Variant does not have `extrapolate` field."),
        }
    }

    /// Function to set extrapolate variable from enum variants
    /// # Arguments
    /// - `new_extrapolate`: updated `extrapolate` variable to replace the current `extrapolate` variable
    pub fn set_extrapolate(&mut self, new_extrapolate: Extrapolate) -> anyhow::Result<()> {
        match self {
            Interpolator::Interp1D(interp) => interp.extrapolate = new_extrapolate,
            Interpolator::Interp2D(interp) => interp.extrapolate = new_extrapolate,
            Interpolator::Interp3D(interp) => interp.extrapolate = new_extrapolate,
            #[cfg(feature = "nd")]
            Interpolator::InterpND(interp) => interp.extrapolate = new_extrapolate,
            _ => anyhow::bail!("Variant does not have `extrapolate` field."),
        }
        Ok(())
    }

    /// Function to get y variable from enum variants
    pub fn y(&self) -> anyhow::Result<Vec<f64>> {
        match self {
            Interpolator::Interp2D(interp) => Ok(interp.y.to_vec()),
            Interpolator::Interp3D(interp) => Ok(interp.y.to_vec()),
            _ => anyhow::bail!("Variant does not have `y` field."),
        }
    }

    /// Function to set y variable from enum variants
    /// # Arguments
    /// - `new_y`: updated `y` variable to replace the current `y` variable
    pub fn set_y(&mut self, new_y: Vec<f64>) -> anyhow::Result<()> {
        match self {
            Interpolator::Interp2D(interp) => interp.set_y(new_y)?,
            Interpolator::Interp3D(interp) => interp.set_y(new_y)?,
            #[cfg(feature = "nd")]
            Interpolator::InterpND(interp) => interp.set_grid_y(new_y)?,
            _ => {
                anyhow::bail!("Variant does not have `y` field (or a grid row 1 (indexed from 0) in the case of InterpND).")
            }
        }
        Ok(())
    }

    /// Function to get f_xy variable from enum variants
    pub fn f_xy(&self) -> anyhow::Result<Vec<Vec<f64>>> {
        match self {
            Interpolator::Interp2D(interp) => Ok(interp.f_xy.to_vec()),
            _ => anyhow::bail!("Variant does not have `f_xy` field."),
        }
    }

    /// Function to set f_xy variable from enum variants
    /// # Arguments
    /// - `new_f_xy`: updated `f_xy` variable to replace the current `f_xy` variable
    pub fn set_f_xy(&mut self, new_f_xy: Vec<Vec<f64>>) -> anyhow::Result<()> {
        match self {
            Interpolator::Interp2D(interp) => interp.set_f_xy(new_f_xy)?,
            _ => anyhow::bail!("Variant does not have `f_xy` field."),
        }
        Ok(())
    }

    /// Function to get z variable from enum variants
    pub fn z(&self) -> anyhow::Result<Vec<f64>> {
        match self {
            Interpolator::Interp3D(interp) => Ok(interp.z.to_vec()),
            _ => anyhow::bail!("Variant does not have `z` field."),
        }
    }

    /// Function to set z variable from enum variants
    /// # Arguments
    /// - `new_z`: updated `z` variable to replace the current `z` variable
    pub fn set_z(&mut self, new_z: Vec<f64>) -> anyhow::Result<()> {
        match self {
            Interpolator::Interp3D(interp) => interp.set_z(new_z)?,
            #[cfg(feature = "nd")]
            Interpolator::InterpND(interp) => interp.set_grid_z(new_z)?,
            _ => {
                anyhow::bail!("Variant does not have `z` field (or a grid row 2 (indexed from 0) in the case of InterpND).")
            }
        }
        Ok(())
    }

    /// Function to get f_xyz variable from enum variants
    pub fn f_xyz(&self) -> anyhow::Result<Vec<Vec<Vec<f64>>>> {
        match self {
            Interpolator::Interp3D(interp) => Ok(interp.f_xyz.to_vec()),
            _ => anyhow::bail!("Variant does not have `f_xyz` field."),
        }
    }

    /// Function to set f_xyz variable from enum variants
    /// # Arguments
    /// - `new_f_xyz`: updated `f_xyz` variable to replace the current `f_xyz` variable
    pub fn set_f_xyz(&mut self, new_f_xyz: Vec<Vec<Vec<f64>>>) -> anyhow::Result<()> {
        match self {
            Interpolator::Interp3D(interp) => interp.set_f_xyz(new_f_xyz)?,
            _ => anyhow::bail!("Variant does not have `f_xyz` field."),
        }
        Ok(())
    }

    /// Function to get grid variable from enum variants
    #[cfg(feature = "nd")]
    pub fn grid(&self) -> anyhow::Result<Vec<Vec<f64>>> {
        match self {
            Interpolator::InterpND(interp) => Ok(interp.grid.to_vec()),
            _ => anyhow::bail!("Variant does not have `grid` field."),
        }
    }

    /// Function to set grid variable from enum variants
    /// # Arguments
    /// - `new_grid`: updated `grid` variable to replace the current `grid` variable
    #[cfg(feature = "nd")]
    pub fn set_grid(&mut self, new_grid: Vec<Vec<f64>>) -> anyhow::Result<()> {
        match self {
            Interpolator::InterpND(interp) => interp.set_grid(new_grid)?,
            _ => anyhow::bail!("Variant does not have `grid` field."),
        }
        Ok(())
    }

    /// Function to get values variable from enum variants
    #[cfg(feature = "nd")]
    pub fn values(&self) -> anyhow::Result<&ndarray::ArrayD<f64>> {
        match self {
            Interpolator::InterpND(interp) => Ok(&interp.values),
            _ => anyhow::bail!("Variant does not have `values` field."),
        }
    }

    /// Function to set values variable from enum variants
    /// # Arguments
    /// - `new_values`: updated `values` variable to replace the current `values` variable
    #[cfg(feature = "nd")]
    pub fn set_values(&mut self, new_values: ndarray::ArrayD<f64>) -> anyhow::Result<()> {
        match self {
            Interpolator::InterpND(interp) => interp.set_values(new_values)?,
            _ => anyhow::bail!("Variant does not have `values` field."),
        }
        Ok(())
    }
}

/// Interpolation strategy.
#[derive(Clone, Debug, PartialEq, Default)]
#[cfg_attr(feature = "serde", Deserialize, Serialize)]
pub enum Strategy {
    /// Linear interpolation: https://en.wikipedia.org/wiki/Linear_interpolation
    #[default]
    Linear,
    /// Left-nearest (previous value) interpolation: https://en.wikipedia.org/wiki/Nearest-neighbor_interpolation
    LeftNearest,
    /// Right-nearest (next value) interpolation: https://en.wikipedia.org/wiki/Nearest-neighbor_interpolation
    RightNearest,
    /// Nearest value (left or right) interpolation: https://en.wikipedia.org/wiki/Nearest-neighbor_interpolation
    Nearest,
}

/// Extrapolation strategy.
///
/// Controls what happens if supplied interpolant point
/// is outside the bounds of the interpolation grid.
#[derive(Clone, Debug, PartialEq, Default)]
#[cfg_attr(feature = "serde", Deserialize, Serialize)]
pub enum Extrapolate {
    /// If interpolant point is beyond the limits of the interpolation grid,
    /// find result via extrapolation using slope of nearby points.  
    /// Only implemented for 1-D linear interpolation.
    Extrapolate,
    /// Restrict interpolant point to the limits of the interpolation grid, using [`f64::clamp`].
    Clamp,
    /// Return an error when interpolant point is beyond the limits of the interpolation grid.
    #[default]
    Error,
}

pub trait InterpMethods {
    // TODO: maybe add `new` to `InterpMethods`
    /// Validate data stored in [Self].  By design, [Self] can be instantiatated
    /// only via [Self::new], which calls this method.
    fn validate(&self) -> Result<(), ValidationError>;
    fn interpolate(&self, point: &[f64]) -> anyhow::Result<f64>;
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
        assert!(interp.interpolate(&[0.]).is_err());
    }
}
