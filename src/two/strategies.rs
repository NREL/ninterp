use super::*;

impl Strategy2D for Linear {
    fn interpolate(&self, data: &Data2D, point: &[f64; 2]) -> Result<f64, InterpolateError> {
        // Extrapolation is checked previously in Interpolator::interpolate,
        // meaning:
        // - point is within grid bounds, or
        // - point is clamped, or
        // - extrapolation is enabled
        let grid = [&data.x, &data.y];
        let lowers: Vec<usize> = (0..2)
            .map(|dim| {
                if &point[dim] < grid[dim].first().unwrap() {
                    0
                } else if &point[dim] > grid[dim].last().unwrap() {
                    grid[dim].len() - 2
                } else {
                    find_nearest_index(grid[dim], point[dim])
                }
            })
            .collect();
        // x
        let x_l = lowers[0];
        let x_u = x_l + 1;
        let x_diff = (point[0] - data.x[x_l]) / (data.x[x_u] - data.x[x_l]);
        // y
        let y_l = lowers[1];
        let y_u = y_l + 1;
        let y_diff = (point[1] - data.y[y_l]) / (data.y[y_u] - data.y[y_l]);
        // interpolate in the x-direction
        let f0 = data.f_xy[x_l][y_l] * (1.0 - x_diff) + data.f_xy[x_u][y_l] * x_diff;
        let f1 = data.f_xy[x_l][y_u] * (1.0 - x_diff) + data.f_xy[x_u][y_u] * x_diff;
        // interpolate in the y-direction
        Ok(f0 * (1.0 - y_diff) + f1 * y_diff)
    }

    /// Returns `true`
    fn allow_extrapolate(&self) -> bool {
        true
    }
}

impl Strategy2D for Nearest {
    fn interpolate(&self, data: &Data2D, point: &[f64; 2]) -> Result<f64, InterpolateError> {
        // x
        let x_l = find_nearest_index(&data.x, point[0]);
        let x_u = x_l + 1;
        let x_diff = (point[0] - data.x[x_l]) / (data.x[x_u] - data.x[x_l]);
        let i = if x_diff < 0.5 { x_l } else { x_u };
        // y
        let y_l = find_nearest_index(&data.y, point[1]);
        let y_u = y_l + 1;
        let y_diff = (point[1] - data.y[y_l]) / (data.y[y_u] - data.y[y_l]);
        let j = if y_diff < 0.5 { y_l } else { y_u };

        Ok(data.f_xy[i][j])
    }

    /// Returns `false`
    fn allow_extrapolate(&self) -> bool {
        false
    }
}
