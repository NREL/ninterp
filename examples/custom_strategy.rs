use ninterp::data::InterpData2D;
use ninterp::prelude::*;
use ninterp::strategy::traits::*;

// Note: ninterp also re-exposes the internally used `ndarray` crate
// `use ninterp::ndarray;`
use ndarray::prelude::*;
use ndarray::{Data, RawDataClone};

// Debug must be derived for custom strategies
#[derive(Debug, Clone)]
struct CustomStrategy;

// Implement strategy for 2-D f32 interpolation
impl<D> Strategy2D<D> for CustomStrategy
where
    // Implement for any 2-D interpolator where the contained type is `f32`
    // e.g. `Array2<f32>`, `ArrayView2<f32>`, `CowArray<<'a, f32>, Ix2>`, etc.
    // For a more generic bound, consider introducing a bound for D::Elem
    // e.g. D::Elem: num_traits::Num + PartialOrd
    D: Data<Elem = f32> + RawDataClone + Clone,
{
    fn interpolate(
        &self,
        _data: &InterpData2D<D>,
        point: &[f32; 2],
    ) -> Result<f32, ninterp::error::InterpolateError> {
        // Dummy interpolation strategy, product of all point components
        // Here we could access the `InterpData2D` instead,
        // but this is just an example.
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
