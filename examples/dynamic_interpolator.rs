use ndarray::prelude::*;

use ninterp::prelude::*;

fn main() {
    // Create `Interpolator` trait object
    let mut boxed: Box<dyn Interpolator<_>> = Box::new(
        Interp2D::new(
            array![0., 1.],
            array![0., 1.],
            array![[2., 4.], [4., 16.]],
            Linear,
            Extrapolate::Enable,
        )
        .unwrap(),
    );
    assert_eq!(boxed.interpolate(&[1.5, -0.5]).unwrap(), -3.5);
    // Change underlying interpolator
    boxed = Box::new(
        Interp1D::new(
            array![0., 1., 2.],
            array![0., 4., 8.],
            Nearest,
            Extrapolate::Error,
        )
        .unwrap(),
    );
    assert_eq!(boxed.interpolate(&[1.75]).unwrap(), 8.)
}
