use super::*;

impl<T> Strategy2D<T> for Linear
where
    T: Num + PartialOrd + Copy + Debug,
{
    fn interpolate(&self, data: &InterpData2D<T>, point: &[T; 2]) -> Result<T, InterpolateError> {
        // Extrapolation is checked previously in Interpolator::interpolate,
        // meaning:
        // - point is within grid bounds, or
        // - point is clamped, or
        // - extrapolation is enabled
        let lowers: Vec<usize> = (0..2)
            .map(|dim| {
                if &point[dim] < data.grid[dim].first().unwrap() {
                    0
                } else if &point[dim] > data.grid[dim].last().unwrap() {
                    data.grid[dim].len() - 2
                } else {
                    find_nearest_index(data.grid[dim].view(), point[dim])
                }
            })
            .collect();
        // x
        let x_l = lowers[0];
        let x_u = x_l + 1;
        let x_diff = (point[0] - data.grid[0][x_l]) / (data.grid[0][x_u] - data.grid[0][x_l]);
        // y
        let y_l = lowers[1];
        let y_u = y_l + 1;
        let y_diff = (point[1] - data.grid[1][y_l]) / (data.grid[1][y_u] - data.grid[1][y_l]);
        // interpolate in the x-direction
        let f0 = data.values[[x_l, y_l]] * (T::one() - x_diff) + data.values[[x_u, y_l]] * x_diff;
        let f1 = data.values[[x_l, y_u]] * (T::one() - x_diff) + data.values[[x_u, y_u]] * x_diff;
        // interpolate in the y-direction
        Ok(f0 * (T::one() - y_diff) + f1 * y_diff)
    }

    /// Returns `true`
    fn allow_extrapolate(&self) -> bool {
        true
    }
}

impl<T> Strategy2D<T> for Nearest
where
    T: Num + PartialOrd + Copy + Debug,
{
    fn interpolate(&self, data: &InterpData2D<T>, point: &[T; 2]) -> Result<T, InterpolateError> {
        // x
        let x_l = find_nearest_index(data.grid[0].view(), point[0]);
        let x_u = x_l + 1;
        let i = if point[0] - data.grid[0][x_l] < data.grid[0][x_u] - point[0] {
            x_l
        } else {
            x_u
        };
        // y
        let y_l = find_nearest_index(data.grid[1].view(), point[1]);
        let y_u = y_l + 1;
        let j = if point[1] - data.grid[1][y_l] < data.grid[1][y_u] - point[1] {
            y_l
        } else {
            y_u
        };

        Ok(data.values[[i, j]])
    }

    /// Returns `false`
    fn allow_extrapolate(&self) -> bool {
        false
    }
}
