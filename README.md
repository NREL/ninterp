# ninterp

[![docs.rs](https://img.shields.io/docsrs/ninterp)](https://docs.rs/ninterp/latest/ninterp) [![Crates.io Version](https://img.shields.io/crates/v/ninterp)](https://crates.io/crates/ninterp) [![GitHub](https://img.shields.io/badge/github-NREL/ninterp-blue)](https://github.com/NREL/ninterp/)

The `ninterp` crate provides [multivariate interpolation](https://en.wikipedia.org/wiki/Multivariate_interpolation#Regular_grid) over rectilinear grids of any dimensionality. A variety of interpolation strategies are implemented, and more are likely to be added.

There are hard-coded interpolators for lower dimensionalities (up to N = 3) for better runtime performance.

```
cargo add ninterp
```

## Feature Flags
- `serde`: support for serde

## Getting Started
A prelude module has been defined: `use ninterp::prelude::*;`.
This exposes the types necessary for usage: [`Interpolator`](https://docs.rs/ninterp/latest/ninterp/enum.Interpolator.html), [`Strategy`](https://docs.rs/ninterp/latest/ninterp/enum.Strategy.html), and [`Extrapolate`](https://docs.rs/ninterp/latest/ninterp/enum.Extrapolate.html).

Interpolation is executed by calling [`Interpolator::interpolate`](https://docs.rs/ninterp/latest/ninterp/enum.Interpolator.html#method.interpolate).
The length of the supplied point slice must be equal to the interpolator dimensionality.

For interpolators of dimensionality N ≥ 1:
- Instantiation is done via the Interpolator enum's `new_*` methods (`new_1d`, `new_2d`, `new_3d`, `new_nd`).
These methods run a validation step that catches any potential errors early, preventing runtime panics.
  - To set or get field values, use the corresponding named methods (`x`, `set_x`, etc.).
- An interpolation [`Strategy`](https://docs.rs/ninterp/latest/ninterp/enum.Strategy.html) (e.g. linear, left-nearest, etc.) must be specified.
Not all interpolation strategies are implemented for every dimensionality.
`Strategy::Linear` and `Strategy::Nearest` are implemented for all dimensionalities.
- An [`Extrapolate`](https://docs.rs/ninterp/latest/ninterp/enum.Extrapolate.html) setting must be specified.
This controls what happens when a point is beyond the range of supplied coordinates.
If you are unsure which variant to choose, `Extrapolate::Error` is likely what you want.
Linear extrapolation is implemented for all dimensionalities.

For 0-D (constant-value) interpolators, instantiate directly, e.g. `Interp0D(0.5)`

### Examples
- [`Interp0D`](https://docs.rs/ninterp/latest/ninterp/enum.Interpolator.html#variant.Interp0D)
- [`Interp1D::new`](https://docs.rs/ninterp/latest/ninterp/enum.Interpolator.html#method.new_1d)
- [`Interp2D::new`](https://docs.rs/ninterp/latest/ninterp/enum.Interpolator.html#method.new_2d)
- [`Interp3D::new`](https://docs.rs/ninterp/latest/ninterp/enum.Interpolator.html#method.new_3d)
- [`InterpND::new`](https://docs.rs/ninterp/latest/ninterp/enum.Interpolator.html#method.new_nd)
