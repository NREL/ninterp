use super::*;

use itertools::Itertools;

fn get_index_permutations(shape: &[usize]) -> Vec<Vec<usize>> {
    if shape.is_empty() {
        return vec![vec![]];
    }
    shape
        .iter()
        .map(|&len| 0..len)
        .multi_cartesian_product()
        .collect()
}

impl StrategyND for Linear {
    fn interpolate(&self, data: &DataND, point: &[f64]) -> Result<f64, InterpolateError> {
        // Dimensionality
        let mut n = data.values.ndim();

        // Point can share up to N values of a grid point, which reduces the problem dimensionality
        // i.e. the point shares one of three values of a 3-D grid point, then the interpolation becomes 2-D at that slice
        // or   if the point shares two of three values of a 3-D grid point, then the interpolation becomes 1-D
        let mut point = point.to_vec();
        let mut grid = data.grid.clone();
        let mut values_view = data.values.view();
        for dim in (0..n).rev() {
            // Range is reversed so that removal doesn't affect indexing
            if let Some(pos) = grid[dim]
                .iter()
                .position(|&grid_point| grid_point == point[dim])
            {
                point.remove(dim);
                grid.remove(dim);
                values_view.index_axis_inplace(Axis(dim), pos);
            }
        }
        if values_view.len() == 1 {
            // Supplied point is coincident with a grid point, so just return the value
            return Ok(values_view.first().copied().unwrap());
        }
        // Simplified dimensionality
        n = values_view.ndim();

        // Extract the lower and upper indices for each dimension,
        // as well as the fraction of how far the supplied point is between the surrounding grid points
        let mut lower_idxs = Vec::with_capacity(n);
        let mut interp_diffs = Vec::with_capacity(n);
        for dim in 0..n {
            // Extrapolation is checked previously in Interpolator::interpolate,
            // meaning:
            // - point is within grid bounds, or
            // - point is clamped, or
            // - extrapolation is enabled
            let lower_idx = if &point[dim] < grid[dim].first().unwrap() {
                0
            } else if &point[dim] > grid[dim].last().unwrap() {
                grid[dim].len() - 2
            } else {
                find_nearest_index(&grid[dim], point[dim])
            };
            let interp_diff = (point[dim] - grid[dim][lower_idx])
                / (grid[dim][lower_idx + 1] - grid[dim][lower_idx]);
            lower_idxs.push(lower_idx);
            interp_diffs.push(interp_diff);
        }
        // `interp_vals` contains all values surrounding the point of interest, starting with shape (2, 2, ...) in N dimensions
        // this gets mutated and reduces in dimension each iteration, filling with the next values to interpolate with
        // this ends up as a 0-dimensional array containing only the final interpolated value
        let mut interp_vals = values_view
            .slice_each_axis(|ax| {
                let lower = lower_idxs[ax.axis.0];
                ndarray::Slice::from(lower..=lower + 1)
            })
            .to_owned();
        let mut index_permutations = get_index_permutations(interp_vals.shape());
        // This loop interpolates in each dimension sequentially
        // each outer loop iteration the dimensionality reduces by 1
        // `interp_vals` ends up as a 0-dimensional array containing only the final interpolated value
        for (dim, diff) in interp_diffs.iter().enumerate() {
            let next_dim = n - 1 - dim;
            let next_shape = vec![2; next_dim];
            // Indeces used for saving results of this dimensions interpolation results
            // assigned to `index_permutations` at end of loop to be used for indexing in next iteration
            let next_idxs = get_index_permutations(&next_shape);
            let mut intermediate_arr = Array::default(next_shape);
            for i in 0..next_idxs.len() {
                // `next_idxs` is always half the length of `index_permutations`
                let l = index_permutations[i].as_slice();
                let u = index_permutations[next_idxs.len() + i].as_slice();
                if dim == 0 && (interp_vals[l].is_nan() || interp_vals[u].is_nan()) {
                    return Err(InterpolateError::NaNError(format!(
                        "\npoint = {point:?},\ngrid = {grid:?},\nvalues = {:?}",
                        data.values
                    )));
                }
                // This calculation happens 2^(n-1) times in the first iteration of the outer loop,
                // 2^(n-2) times in the second iteration, etc.
                intermediate_arr[next_idxs[i].as_slice()] =
                    interp_vals[l] * (1.0 - diff) + interp_vals[u] * diff;
            }
            index_permutations = next_idxs;
            interp_vals = intermediate_arr;
        }

        // return the only value contained within the 0-dimensional array
        Ok(interp_vals.first().copied().unwrap())
    }

    fn allow_extrapolate(&self) -> bool {
        true
    }
}

impl StrategyND for Nearest {
    fn interpolate(&self, data: &DataND, point: &[f64]) -> Result<f64, InterpolateError> {
        // Dimensionality
        let mut n = data.values.ndim();

        // Point can share up to N values of a grid point, which reduces the problem dimensionality
        // i.e. the point shares one of three values of a 3-D grid point, then the interpolation becomes 2-D at that slice
        // or   if the point shares two of three values of a 3-D grid point, then the interpolation becomes 1-D
        let mut point = point.to_vec();
        let mut grid = data.grid.clone();
        let mut values_view = data.values.view();
        for dim in (0..n).rev() {
            // Range is reversed so that removal doesn't affect indexing
            if let Some(pos) = grid[dim]
                .iter()
                .position(|&grid_point| grid_point == point[dim])
            {
                point.remove(dim);
                grid.remove(dim);
                values_view.index_axis_inplace(Axis(dim), pos);
            }
        }
        if values_view.len() == 1 {
            // Supplied point is coincident with a grid point, so just return the value
            return Ok(values_view.first().copied().unwrap());
        }
        // Simplified dimensionality
        n = values_view.ndim();

        // Extract the lower and upper indices for each dimension,
        // as well as the fraction of how far the supplied point is between the surrounding grid points
        let mut lower_idxs = Vec::with_capacity(n);
        let mut interp_diffs = Vec::with_capacity(n);
        for dim in 0..n {
            let lower_idx = find_nearest_index(&grid[dim], point[dim]);
            let interp_diff = (point[dim] - grid[dim][lower_idx])
                / (grid[dim][lower_idx + 1] - grid[dim][lower_idx]);
            lower_idxs.push(lower_idx);
            interp_diffs.push(interp_diff);
        }
        // `interp_vals` contains all values surrounding the point of interest, starting with shape (2, 2, ...) in N dimensions
        // this gets mutated and reduces in dimension each iteration, filling with the next values to interpolate with
        // this ends up as a 0-dimensional array containing only the final interpolated value
        let mut interp_vals = values_view
            .slice_each_axis(|ax| {
                let lower = lower_idxs[ax.axis.0];
                ndarray::Slice::from(lower..=lower + 1)
            })
            .to_owned();
        let mut index_permutations = get_index_permutations(interp_vals.shape());
        // This loop interpolates in each dimension sequentially
        // each outer loop iteration the dimensionality reduces by 1
        // `interp_vals` ends up as a 0-dimensional array containing only the final interpolated value
        for (dim, diff) in interp_diffs.iter().enumerate() {
            let next_dim = n - 1 - dim;
            let next_shape = vec![2; next_dim];
            // Indeces used for saving results of this dimensions interpolation results
            // assigned to `index_permutations` at end of loop to be used for indexing in next iteration
            let next_idxs = get_index_permutations(&next_shape);
            let mut intermediate_arr = Array::default(next_shape);
            for i in 0..next_idxs.len() {
                // `next_idxs` is always half the length of `index_permutations`
                let l = index_permutations[i].as_slice();
                let u = index_permutations[next_idxs.len() + i].as_slice();
                if dim == 0 && (interp_vals[l].is_nan() || interp_vals[u].is_nan()) {
                    return Err(InterpolateError::NaNError(format!(
                        "\npoint = {point:?},\ngrid = {grid:?},\nvalues = {:?}",
                        data.values
                    )));
                }
                // This calculation happens 2^(n-1) times in the first iteration of the outer loop,
                // 2^(n-2) times in the second iteration, etc.
                intermediate_arr[next_idxs[i].as_slice()] = if diff < &0.5 {
                    interp_vals[l]
                } else {
                    interp_vals[u]
                };
            }
            index_permutations = next_idxs;
            interp_vals = intermediate_arr;
        }

        // return the only value contained within the 0-dimensional array
        Ok(interp_vals.first().copied().unwrap())
    }

    fn allow_extrapolate(&self) -> bool {
        false
    }
}
