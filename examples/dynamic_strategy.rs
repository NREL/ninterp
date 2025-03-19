use ndarray::prelude::*;

use ninterp::prelude::*;

fn main() {
    // Create mutable interpolator
    let mut interp = Interp1D::new(
        array![0., 1., 2.],
        array![0., 3., 6.],
        // Provide the strategy as a trait object
        Box::new(strategy::Linear) as Box<dyn strategy::traits::Strategy1D<_>>,
        Extrapolate::Error,
    )
    .unwrap();
    assert_eq!(interp.interpolate(&[1.75]).unwrap(), 5.25);
    // Change strategy to `Nearest`
    interp.set_strategy(Box::new(strategy::Nearest)).unwrap();
    assert_eq!(interp.interpolate(&[1.75]).unwrap(), 6.);
}
