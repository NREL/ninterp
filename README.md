# ninterp

[![docs.rs](https://img.shields.io/docsrs/ninterp)](https://docs.rs/ninterp/latest/ninterp) [![Crates.io Version](https://img.shields.io/crates/v/ninterp)](https://crates.io/crates/ninterp) [![GitHub](https://img.shields.io/badge/github-NREL/ninterp-blue)](https://github.com/NREL/ninterp/)

The `ninterp` crate provides [multivariate interpolation](https://en.wikipedia.org/wiki/Multivariate_interpolation#Regular_grid) over a regular, sorted, nonrepeating grid of any dimensionality. A variety of interpolation strategies are implemented, however more are likely to be added. Extrapolation beyond the range of the supplied coordinates is supported for 1-D linear interpolators, using the slope of the nearby points.

There are hard-coded interpolators for lower dimensionalities (up to N = 3) for better runtime performance.

All interpolation is handled through instances of the [Interpolator](https://docs.rs/ninterp/latest/ninterp/enum.Interpolator.html) enum, with the selected tuple variant containing relevant data. Interpolation is executed by calling [Interpolator::interpolate](https://docs.rs/ninterp/latest/ninterp/enum.Interpolator.html#method.interpolate).

## Feature Flags
- `serde`: support for serde

## Getting Started

See the [Interpolator](https://docs.rs/ninterp/latest/ninterp/enum.Interpolator.html) enum documentation for examples and notes on usage.
