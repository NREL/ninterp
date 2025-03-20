# ninterp

[![docs.rs](https://img.shields.io/docsrs/ninterp)](https://docs.rs/ninterp/latest/ninterp) [![Crates.io Version](https://img.shields.io/crates/v/ninterp)](https://crates.io/crates/ninterp) [![GitHub](https://img.shields.io/badge/github-NREL/ninterp-blue)](https://github.com/NREL/ninterp/)

The `ninterp` crate provides [multivariate interpolation](https://en.wikipedia.org/wiki/Multivariate_interpolation#Regular_grid) over rectilinear grids of any dimensionality.

There are hard-coded interpolators for lower dimensionalities (up to N = 3) for better runtime performance.
All interpolators work with both owned and borrowed arrays (array views) of various types.

A variety of interpolation strategies are implemented and exposed in the `prelude` module.
Custom interpolation strategies can be defined in downstream crates.

```
cargo add ninterp
```

#### Cargo Features
- `serde`: support for serde ([caveat](https://github.com/NREL/ninterp/issues/5))
  ```
  cargo add ninterp --features serde
  ```

## Examples
See examples in `new` method documentation:
- [`Interp0D::new`](https://docs.rs/ninterp/latest/ninterp/interpolator/struct.Interp0D.html#method.new)
- [`Interp1D::new`](https://docs.rs/ninterp/latest/ninterp/interpolator/struct.Interp1D.html#method.new)
- [`Interp2D::new`](https://docs.rs/ninterp/latest/ninterp/interpolator/struct.Interp2D.html#method.new)
- [`Interp3D::new`](https://docs.rs/ninterp/latest/ninterp/interpolator/struct.Interp3D.html#method.new)
- [`InterpND::new`](https://docs.rs/ninterp/latest/ninterp/interpolator/struct.InterpND.html#method.new)

Also see the [`examples`](examples) directory for advanced examples:
- Swapping strategies at runtime: [`dynamic_strategy.rs`](examples/dynamic_strategy.rs)
  - Using strategy enums `strategy::enums::Strategy1DEnum`/etc
    - Compatible with `serde`
    - Incompatible with custom strategies
  - Using dynamic dispatch `Box<dyn Strategy1D>`/etc.)
    - Incompatible with `serde`
    - Compatible with custom strategies
    - Runtime cost

- Interpolator dynamic dispatch using `Box<dyn Interpolator>`: [`dynamic_interpolator.rs`](examples/dynamic_interpolator.rs)

- Defining custom strategies: [`custom_strategy.rs`](examples/custom_strategy.rs)

- Using transmutable (transparent) types, such as `uom::si::Quantity`: [`uom.rs`](examples/uom.rs)

## Overview
A prelude module has been defined: 
```rust
use ninterp::prelude::*;
```

This exposes all strategies and a variety of interpolators:
- [`Interp1D`](https://docs.rs/ninterp/latest/ninterp/interpolator/struct.Interp1D.html)
- [`Interp2D`](https://docs.rs/ninterp/latest/ninterp/interpolator/struct.Interp2D.html)
- [`Interp3D`](https://docs.rs/ninterp/latest/ninterp/interpolator/struct.Interp3D.html)
- [`InterpND`](https://docs.rs/ninterp/latest/ninterp/interpolator/struct.InterpND.html)

There is also a constant-value 'interpolator':
[`Interp0D`](https://docs.rs/ninterp/latest/ninterp/interpolator/struct.Interp0D.html).
This is useful when working with a `Box<dyn Interpolator>`

Instantiation is done by calling an interpolator's `new` method.
For dimensionalities N ≥ 1, this executes a validation step, preventing runtime panics.
After editing interpolator data,
call the InterpData's `validate` method
or [`Interpolator::validate`](https://docs.rs/ninterp/latest/ninterp/interpolator/trait.Interpolator.html#tymethod.validate)
to rerun these checks.

To change the extrapolation setting, call `set_extrapolate`.

To change the interpolation strategy,
supply a `Box<dyn Strategy1D>`/etc. upon instantiation,
and call `set_strategy`.

### Strategies
An interpolation strategy (e.g.
[`Linear`](https://docs.rs/ninterp/latest/ninterp/strategy/struct.Linear.html),
[`Nearest`](https://docs.rs/ninterp/latest/ninterp/strategy/struct.Nearest.html),
[`LeftNearest`](https://docs.rs/ninterp/latest/ninterp/strategy/struct.LeftNearest.html),
[`RightNearest`](https://docs.rs/ninterp/latest/ninterp/strategy/struct.RightNearest.html))
must be specified.
Not all interpolation strategies are implemented for every dimensionality.
`Linear` and `Nearest` are implemented for all dimensionalities.

Custom strategies can be defined. See
[`examples/custom_strategy.rs`](examples/custom_strategy.rs)
for an example.

### Extrapolation
An [`Extrapolate`](https://docs.rs/ninterp/latest/ninterp/interpolator/enum.Extrapolate.html)
setting must be provided in the `new` method.
This controls what happens when a point is beyond the range of supplied coordinates.
The following settings are applicable for all interpolators:
- `Extrapolate::Fill(T)`
- `Extrapolate::Clamp`
- `Extrapolate::Wrap`
- `Extrapolate::Error`

`Extrapolate::Enable` is valid for `Linear` for all dimensionalities.

If you are unsure which variant to choose, `Extrapolate::Error` is likely what you want.

### Interpolation
Interpolation is executed by calling [`Interpolator::interpolate`](https://docs.rs/ninterp/latest/ninterp/interpolator/trait.Interpolator.html#tymethod.interpolate).

The length of the interpolant point slice must be equal to the interpolator dimensionality.
The interpolator dimensionality can be retrieved by calling [`Interpolator::ndim`](https://docs.rs/ninterp/latest/ninterp/interpolator/trait.Interpolator.html#tymethod.ndim).
