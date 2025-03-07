use ndarray::prelude::*;

use ninterp::prelude::*;
use ninterp::strategy::*;

// Debug must be derived for custom strategies
#[derive(Debug)]
struct CustomStrategy;

// Implement strategy for 2-D interpolation
impl Strategy2D for CustomStrategy {
    fn interpolate(
        &self,
        _data: &InterpData2D,
        point: &[f64; 2],
    ) -> Result<f64, ninterp::error::InterpolateError> {
        // Dummy interpolation strategy, product of all point components
        Ok(point.iter().fold(1., |acc, x| acc * x))
    }

    // Disallow extrapolation.
    //
    // Returning `false` will mean a combination of
    // `Extrapolate::Enable` and `CustomStrategy` will fail on validation.
    //
    // Only set this to `true` if the `interpolate` implementation provisions for extrapolation.
    fn allow_extrapolate(&self) -> bool {
        false
    }
}

fn main() {
    let interp = Interp2D::new(
        array![0., 2., 4.],
        array![0., 4., 8.],
        array![[0., 0., 0.], [0., 0., 0.], [0., 0., 0.]],
        CustomStrategy,
        Extrapolate::Error,
    )
    .unwrap();
    // 2 * 3 == 6
    assert_eq!(interp.interpolate(&[2., 3.]).unwrap(), 6.);
}
