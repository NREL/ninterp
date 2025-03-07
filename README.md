# ninterp

[![docs.rs](https://img.shields.io/docsrs/ninterp)](https://docs.rs/ninterp/latest/ninterp) [![Crates.io Version](https://img.shields.io/crates/v/ninterp)](https://crates.io/crates/ninterp) [![GitHub](https://img.shields.io/badge/github-NREL/ninterp-blue)](https://github.com/NREL/ninterp/)

The `ninterp` crate provides [multivariate interpolation](https://en.wikipedia.org/wiki/Multivariate_interpolation#Regular_grid) over rectilinear grids of any dimensionality.

There are hard-coded interpolators for lower dimensionalities (up to N = 3) for better runtime performance.

A variety of interpolation strategies are implemented and exposed in the `prelude` module.
Custom interpolation strategies can be defined in downstream crates.

```
cargo add ninterp
```

## Feature Flags
- `serde`: support for serde

## Getting Started
A prelude module has been defined: `use ninterp::prelude::*;`

This exposes a variety of interpolators:
- `Interp1`
- `Interp2`
- `Interp3`
- `InterpN`

There is also a constant-value 'interpolator':
`Interp0`.
This is useful when working with a `Box<dyn Interpolator>`

Instantiation is done by calling an interpolator's `new` method.
For dimensionality N ≥ 1, this executes a validation step, preventing runtime panics.
When manually editing interpolator data, call `Interpolator::validate` to rerun these checks.
Utilize `set_strategy` and `set_extrapolate` methods to change the strategy and extrapolate setting.

### Strategies
An interpolation strategy (e.g. `Linear`, `Nearest`, `LeftNearest`, `RightNearest`) must be specified.
Not all interpolation strategies are implemented for every dimensionality.
`Linear` and `Nearest` are implemented for all dimensionalities.

Custom strategies can be defined. See `examples/custom_strategy.rs` for an example.


### Extrapolation
An `Extrapolate` setting must be specified.
This controls what happens when a point is beyond the range of supplied coordinates.
The following setttings are applicable for all interpolators:
- `Extrapolate::Fill(f64)`
- `Extrapolate::Clamp`
- `Extrapolate::Error`

`Extrapolate::Enable` is valid for `Linear` in all dimensionalities.

If you are unsure which variant to choose, `Extrapolate::Error` is likely what you want.

### Interpolation
Interpolation is executed by calling `interpolate`.
The length of the interpolant point slice must be equal to the interpolator dimensionality.
The interpolator dimensionality can be retrieved by calling `Interpolator::ndim`.

### Examples
See `new` method documentation:
- `Interp0D::new`
- `Interp1D::new`
- `Interp2D::new`
- `Interp3D::new`
- `InterpND::new`

See the `examples` directory for advanced examples.

## Strategy dynamic dispatch
By default, construction of interpolators uses *static dispatch*,
meaning strategy concrete types are determined at compilation.
This gives increased performance at the cost of runtime flexibility.
To enable swapping strategies after instantiation,
use *dynamic dispatch* by providing a trait object `Box<dyn Trait>` to the constructor.

See `examples/dynamic_strategy.rs` for an example.