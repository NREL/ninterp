# ninterp

[![docs.rs](https://img.shields.io/docsrs/ninterp)](https://docs.rs/ninterp/latest/ninterp) [![Crates.io Version](https://img.shields.io/crates/v/ninterp)](https://crates.io/crates/ninterp) [![GitHub](https://img.shields.io/badge/github-NREL/ninterp-blue)](https://github.com/NREL/ninterp/)

The `ninterp` crate provides [multivariate interpolation](https://en.wikipedia.org/wiki/Multivariate_interpolation#Regular_grid) over rectilinear grids of any dimensionality. A variety of interpolation strategies are implemented, however more are likely to be added. Extrapolation beyond the range of the supplied coordinates is supported for 1-D linear interpolators, using the slope of the nearby points.

There are hard-coded interpolators for lower dimensionalities (up to N = 3) for better runtime performance.

```
cargo add ninterp
```

## Feature Flags
- `serde`: support for serde

## Getting Started
A prelude module has been defined: `use ninterp::prelude::*;`.
This exposes the types necessary for usage: `Interpolator`, `Strategy`, `Extrapolate`, and the trait `InterpMethods`.

All interpolation is handled through instances of the [`Interpolator`](https://docs.rs/ninterp/latest/ninterp/enum.Interpolator.html) enum.

Interpolation is executed by calling [`Interpolator::interpolate`](https://docs.rs/ninterp/latest/ninterp/enum.Interpolator.html#method.interpolate).
The length of the supplied point slice must be equal to the interpolator dimensionality.
The interpolator dimensionality can be retrieved by calling [`Interpolator::ndim`](https://docs.rs/ninterp/latest/ninterp/enum.Interpolator.html#method.ndim).

### Note
For interpolators of dimensionality N ≥ 1:
- Instantiation is done via the Interpolator enum's `new_*` methods (`new_1d`, `new_2d`, `new_3d`, `new_nd`).
These methods run a validation step that catches any potential errors early, preventing runtime panics.
  - To set or get field values, use the corresponding named methods (`x`, `set_x`, etc.).
- An interpolation [`Strategy`](https://docs.rs/ninterp/latest/ninterp/enum.Strategy.html) (e.g. linear, left-nearest, etc.) must be specified.
Not all interpolation strategies are implemented for every dimensionality.
`Strategy::Linear` is implemented for all dimensionalities.
- An [`Extrapolate`](https://docs.rs/ninterp/latest/ninterp/enum.Extrapolate.html) setting must be specified.
This controls what happens when a point is beyond the range of supplied coordinates.
If you are unsure which variant to choose, `Extrapolate::Error` is likely what you want.

For 0-D (constant-value) interpolators, instantiate directly, e.g. `Interpolator::Interp0D(0.5)`

### Examples
- [`Interpolator::Interp0D`](https://docs.rs/ninterp/latest/ninterp/enum.Interpolator.html#variant.Interp0D)
- [`Interpolator::new_1d`](https://docs.rs/ninterp/latest/ninterp/enum.Interpolator.html#method.new_1d)
- [`Interpolator::new_2d`](https://docs.rs/ninterp/latest/ninterp/enum.Interpolator.html#method.new_2d)
- [`Interpolator::new_3d`](https://docs.rs/ninterp/latest/ninterp/enum.Interpolator.html#method.new_3d)
- [`Interpolator::new_nd`](https://docs.rs/ninterp/latest/ninterp/enum.Interpolator.html#method.new_nd)
