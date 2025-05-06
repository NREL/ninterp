//! The `ninterp` crate provides
//! [multivariate interpolation](https://en.wikipedia.org/wiki/Multivariate_interpolation#Regular_grid)
//! over rectilinear grids of any dimensionality.
//!
//! There are hard-coded interpolators for lower dimensionalities (up to N = 3) for better runtime performance.
//! All interpolators work with both owned and borrowed arrays (array views) of various types.
//!
//! A variety of interpolation strategies are implemented and exposed in the [`prelude`] module.
//! Custom interpolation strategies can be defined in downstream crates.
//!
//! ```text
//! cargo add ninterp
//! ```
//!
//! ### Cargo Features
//! - `serde`: support for [`serde`](https://crates.io/crates/serde) 1.x
//!   ```text
//!   cargo add ninterp --features serde
//!   ```
//!
//! # Examples
//! See examples in `new` method documentation:
//! - [`Interp0D::new`](`interpolator::Interp0D::new`)
//! - [`Interp1D::new`](`interpolator::Interp1D::new`)
//! - [`Interp2D::new`](`interpolator::Interp2D::new`)
//! - [`Interp3D::new`](`interpolator::Interp3D::new`)
//! - [`InterpND::new`](`interpolator::InterpND::new`)
//!
//! Also see the `examples` directory for advanced examples:
//! - **`dynamic_strategy.rs`**
//!
//!   Swapping strategies at runtime
//!   - Using strategy enums ([`strategy::enums::Strategy1DEnum`]/etc.)
//!     - Compatible with `serde`
//!     - Incompatible with custom strategies
//!   - Using [`Box<dyn Strategy1D>`]/etc. (dynamic dispatch)
//!     - Incompatible with `serde`
//!     - Compatible with custom strategies
//!     - Runtime cost
//!
//! - **`dynamic_interpolator.rs`**
//!
//!   Swapping interpolators at runtime
//!   - Using [`InterpolatorEnum`](interpolator::enums::InterpolatorEnum)
//!     - Compatible with `serde`
//!     - Incompatible with custom strategies
//!   - Using [`Box<dyn Interpolator>`] (dynamic dispatch)
//!     - Incompatible with `serde`
//!     - Compatible with custom strategies
//!     - Runtime cost
//!
//! - **`custom_strategy.rs`**
//!
//!   Defining custom strategies
//!
//! - **`uom.rs`**
//!
//!   Using transmutable (transparent) types, such as [`uom::si::Quantity`](https://docs.rs/uom/0.36.0/uom/si/struct.Quantity.html)
//!
//! # Overview
//! A [`prelude`] module has been defined:
//! ```rust,text
//! use ninterp::prelude::*;
//! ```
//!
//! This exposes all strategies and a variety of interpolators:
//! - [`Interp1D`](`interpolator::Interp1D`)
//! - [`Interp2D`](`interpolator::Interp2D`)
//! - [`Interp3D`](`interpolator::Interp3D`)
//! - [`InterpND`](`interpolator::InterpND`)
//!
//! There is also a constant-value 'interpolator':
//! [`Interp0D`](`interpolator::Interp0D`).
//! This is useful when working with an [`InterpolatorEnum`](enums::InterpolatorEnum) or [`Box<dyn Interpolator>`]
//!
//! Instantiation is done by calling an interpolator's `new` method.
//! For dimensionalities N ≥ 1, this executes a validation step, preventing runtime panics.
//! After editing interpolator data,
//! call the InterpData's `validate` method
//! or [`Interpolator::validate`]
//! to rerun these checks.
//!
//! To change the extrapolation setting, call `set_extrapolate`.
//!
//! To change the interpolation strategy,
//! supply a [`Strategy1DEnum`](strategy::enums::Strategy1DEnum)/etc. or [`Box<dyn Strategy1D>`]/etc. upon instantiation,
//! and call `set_strategy`.
//!
//! ## Strategies
//! An interpolation strategy (e.g.
//!   [`Linear`](strategy::Linear),
//!   [`Nearest`](strategy::Nearest),
//!   [`LeftNearest`](strategy::LeftNearest),
//!   [`RightNearest`](strategy::RightNearest))
//! must be specified.
//! Not all interpolation strategies are implemented for every dimensionality.
//! [`Linear`](strategy::Linear) and [`Nearest`](strategy::Nearest) are implemented for all dimensionalities.
//!
//! Custom strategies can be defined. See `examples/custom_strategy.rs` for an example.
//!
//! ## Extrapolation
//! An [`Extrapolate`] setting must be provided in the `new` method.
//! This controls what happens when a point is beyond the range of supplied coordinates.
//! The following settings are applicable for all interpolators:
//! - [`Extrapolate::Fill(T)`](`Extrapolate::Fill`)
//! - [`Extrapolate::Clamp`]
//! - [`Extrapolate::Wrap`]
//! - [`Extrapolate::Error`]
//!
//! [`Extrapolate::Enable`] is valid for [`Linear`](strategy::Linear) for all dimensionalities.
//!
//! If you are unsure which variant to choose, [`Extrapolate::Error`] is likely what you want.
//!
//! ## Interpolation
//! Interpolation is executed by calling [`Interpolator::interpolate`].
//!
//! The length of the interpolant point slice must be equal to the interpolator dimensionality.
//! The interpolator dimensionality can be retrieved by calling [`Interpolator::ndim`].
//!
//! # Using Owned and Borrowed (Viewed) Data
//! All interpolators work with both owned and borrowed data.
//! This is accomplished by the generic `D`, which has a bound on the [`ndarray::Data`] trait.
//!
//! Type aliases are provided in the [`prelude`] for convenience, e.g. for 1-D:
//! - [`Interp1DOwned`](`interpolator::Interp1DOwned`)
//!   - Data is *owned* by the interpolator object
//!   - Useful for struct fields
//!   ```rust
//!   use ndarray::prelude::*;
//!   use ninterp::prelude::*;
//!   let interp: Interp1DOwned<f64, _> = Interp1D::new(
//!       array![0.0, 1.0, 2.0, 3.0],
//!       array![0.0, 1.0, 4.0, 9.0],
//!       strategy::Linear,
//!       Extrapolate::Error,
//!   )
//!   .unwrap();
//!   ```
//! - [`Interp1DViewed`](`interpolator::Interp1DViewed`)
//!   - Data is *borrowed* by the interpolator object
//!   - Use when interpolator data should be owned by another object
//!   ```rust
//!   use ndarray::prelude::*;
//!   use ninterp::prelude::*;
//!   let x = array![0.0, 1.0, 2.0, 3.0];
//!   let f_x = array![0.0, 1.0, 4.0, 9.0];
//!   let interp: Interp1DViewed<&f64, _> = Interp1D::new(
//!       x.view(),
//!       f_x.view(),
//!       strategy::Linear,
//!       Extrapolate::Error,
//!   )
//!   .unwrap();
//!   ```
//!
//! Typically, the compiler can determine concrete types using the arguments provided to `new` methods.
//! Examples throughout this crate have type annotions for clarity purposes; they are often unnecessary.

/// The `prelude` module exposes a variety of types:
/// - All interpolator structs:
///   - [`Interp0D`](`interpolator::Interp0D`)
///   - [`Interp1D`](`interpolator::Interp1D`)
///   - [`Interp2D`](`interpolator::Interp2D`)
///   - [`Interp3D`](`interpolator::Interp3D`)
///   - [`InterpND`](`interpolator::InterpND`)
///   - A `serde`-compatible interpolator enum [`InterpolatorEnum`](`interpolator::enums::InterpolatorEnum`)
///   - `Owned` and `Viewed` type aliases for all of the above
/// - Their common trait: [`Interpolator`]
/// - The [`strategy`] mod, containing pre-defined interpolation strategies:
///   - [`strategy::Linear`]
///   - [`strategy::Nearest`]
///   - [`strategy::LeftNearest`]
///   - [`strategy::RightNearest`]
///   - `serde`-compatible strategy enums: [`strategy::enums::Strategy1DEnum`]/etc.
/// - The extrapolation setting enum: [`Extrapolate`]
pub mod prelude {
    pub use crate::strategy;

    pub use crate::interpolator::{Extrapolate, Interpolator};

    pub use crate::interpolator::Interp0D;
    pub use crate::interpolator::{Interp1D, Interp1DOwned, Interp1DViewed};
    pub use crate::interpolator::{Interp2D, Interp2DOwned, Interp2DViewed};
    pub use crate::interpolator::{Interp3D, Interp3DOwned, Interp3DViewed};
    pub use crate::interpolator::{InterpND, InterpNDOwned, InterpNDViewed};

    pub use crate::interpolator::enums::{
        InterpolatorEnum, InterpolatorEnumOwned, InterpolatorEnumViewed,
    };
}

pub mod error;
pub mod strategy;

pub mod interpolator;
pub use interpolator::data;
pub(crate) use interpolator::data::*;
pub(crate) use interpolator::*;

pub(crate) use error::*;
pub(crate) use strategy::traits::*;

pub(crate) use std::fmt::Debug;

pub use ndarray;
pub(crate) use ndarray::prelude::*;
pub(crate) use ndarray::{Data, Ix, RawDataClone};

pub use num_traits;
pub(crate) use num_traits::{clamp, Euclid, Num, One};

pub(crate) use dyn_clone::*;

#[cfg(feature = "serde")]
pub(crate) use ndarray::{DataOwned, IntoDimension};
#[cfg(feature = "serde")]
pub(crate) use serde::{Deserialize, Serialize};
#[cfg(feature = "serde")]
pub(crate) use serde_unit_struct::{Deserialize_unit_struct, Serialize_unit_struct};

#[cfg(test)]
/// Alias for [`approx::assert_abs_diff_eq`] with `epsilon = 1e-6`
macro_rules! assert_approx_eq {
    ($a:expr, $b:expr $(,)?) => {
        approx::assert_abs_diff_eq!($a, $b, epsilon = 1e-6)
    };
    ($a:expr, $b:expr, $eps:expr $(,)?) => {
        approx::assert_abs_diff_eq!($a, $b, epsilon = $eps)
    };
}
#[cfg(test)]
pub(crate) use assert_approx_eq;

/// Wrap value around data bounds.
/// Assumes `min` < `max`.
pub(crate) fn wrap<T: Num + Euclid + Copy>(input: T, min: T, max: T) -> T {
    min + (input - min).rem_euclid(&(max - min))
}

#[cfg(test)]
mod tests {
    use super::wrap;

    #[test]
    fn test_wrap() {
        assert_eq!(wrap(-3, -2, 5), 4);
        assert_eq!(wrap(3, -2, 5), 3);
        assert_eq!(wrap(6, -2, 5), -1);
        assert_eq!(wrap(5, 0, 10), 5);
        assert_eq!(wrap(11, 0, 10), 1);
        assert_eq!(wrap(-3, 0, 10), 7);
        assert_eq!(wrap(-11, 0, 10), 9);
        assert_eq!(wrap(-0.1, -2., -1.), -1.1);
        assert_eq!(wrap(-0., -2., -1.), -2.0);
        assert_eq!(wrap(0.1, -2., -1.), -1.9);
        assert_eq!(wrap(-0.5, -1., 1.), -0.5);
        assert_eq!(wrap(0., -1., 1.), 0.);
        assert_eq!(wrap(0.5, -1., 1.), 0.5);
        assert_eq!(wrap(0.8, -1., 1.), 0.8);
    }
}

#[cfg(feature = "serde")]
mod serde_arrays {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S, D>(grid: &[ArrayBase<D, Ix1>], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        D: Data + RawDataClone + Clone,
        D::Elem: Serialize + Clone,
    {
        let vecs: Vec<Vec<D::Elem>> = grid.iter().map(|arr| arr.to_vec()).collect();
        vecs.serialize(serializer)
    }

    pub fn deserialize<'de, D, De>(deserializer: De) -> Result<Vec<ArrayBase<D, Ix1>>, De::Error>
    where
        De: Deserializer<'de>,
        D: DataOwned + RawDataClone,
        D::Elem: Deserialize<'de> + Clone,
    {
        let vecs: Vec<Vec<D::Elem>> = Vec::deserialize(deserializer)?;
        let arrays = vecs
            .into_iter()
            .map(|v| ArrayBase::<D, Ix1>::from_vec(v))
            .collect();
        Ok(arrays)
    }
}

#[cfg(feature = "serde")]
mod serde_arrays_2 {
    use super::*;
    use serde::de::{Deserializer, Error as DeError, SeqAccess, Visitor};
    use serde::ser::{SerializeSeq, Serializer};
    use std::marker::PhantomData;

    pub fn serialize<S, D, const N: usize>(
        grid: &[ArrayBase<D, Ix1>; N],
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        D: Data + RawDataClone + Clone,
        D::Elem: Serialize + Clone,
    {
        let vecs: [Vec<D::Elem>; N] = std::array::from_fn(|i| grid[i].to_vec());
        let mut seq = serializer.serialize_seq(Some(N))?;
        for vec in &vecs {
            seq.serialize_element(vec)?;
        }
        seq.end()
    }

    pub fn deserialize<'de, D, De, const N: usize>(
        deserializer: De,
    ) -> Result<[ArrayBase<D, Ix1>; N], De::Error>
    where
        De: Deserializer<'de>,
        D: DataOwned,
        D::Elem: Deserialize<'de> + Clone,
    {
        struct ArrayVisitor<D, const N: usize>(PhantomData<D>);

        impl<'de, D, const N: usize> Visitor<'de> for ArrayVisitor<D, N>
        where
            D: DataOwned,
            D::Elem: Deserialize<'de> + Clone,
        {
            type Value = [ArrayBase<D, Ix1>; N];

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str(&format!("an array of {} arrays", N))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                // Create a Vec and then try to convert to array
                let mut arrays = Vec::with_capacity(N);

                // Handle either format (Vec<Vec<T>> or Vec<ArrayBase>)
                for _ in 0..N {
                    // Try to deserialize as Vec<T> first
                    if let Ok(vec) = seq.next_element::<Vec<D::Elem>>() {
                        if let Some(vec) = vec {
                            arrays.push(ArrayBase::<D, Ix1>::from_vec(vec));
                            continue;
                        }
                    }

                    // Then try as ArrayBase
                    if let Ok(arr) = seq.next_element::<ArrayBase<D, Ix1>>() {
                        if let Some(arr) = arr {
                            arrays.push(
                                ArrayBase::<D, Ix1>::from_shape_vec(arr.len(), arr.to_vec())
                                    .map_err(|e| DeError::custom(format!("Shape error: {}", e)))?,
                            );
                            continue;
                        }
                    }

                    // If we get here, we didn't find a valid element
                    return Err(DeError::custom(format!(
                        "Expected {} arrays, found fewer",
                        N
                    )));
                }

                // Convert Vec to fixed-size array
                arrays
                    .try_into()
                    .map_err(|_| DeError::custom(format!("Expected array of length {}", N)))
            }
        }

        deserializer.deserialize_seq(ArrayVisitor::<D, N>(PhantomData))
    }
}
