use ndarray::prelude::*;

use ninterp::prelude::*;
use ninterp::strategy::Strategy1D;

fn main() {
    // Create mutable interpolator
    let mut interp = Interp1D::new(
        array![0., 1., 2.],
        array![0., 3., 6.],
        // Provide the strategy as a trait object
        Box::new(Linear) as Box<dyn Strategy1D>,
        Extrapolate::Error,
    )
    .unwrap();
    assert_eq!(interp.interpolate(&[1.75]).unwrap(), 5.25);
    // Change strategy to `Nearest`
    interp.set_strategy(Box::new(Nearest)).unwrap();
    assert_eq!(interp.interpolate(&[1.75]).unwrap(), 6.);
}
