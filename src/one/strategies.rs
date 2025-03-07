use super::*;

impl Strategy1D for Linear {
    fn interpolate(&self, data: &InterpData1D, point: &[f64; 1]) -> Result<f64, InterpolateError> {
        if let Some(i) = data.grid[0].iter().position(|&x_val| x_val == point[0]) {
            return Ok(data.values[i]);
        }
        // Extrapolation is checked previously in `Interpolator::interpolate`,
        // meaning:
        // - point is within grid bounds, or
        // - point is clamped, or
        // - extrapolation is enabled
        let x_l = if &point[0] < data.grid[0].first().unwrap() {
            0
        } else if &point[0] > data.grid[0].last().unwrap() {
            data.grid[0].len() - 2
        } else {
            find_nearest_index(data.grid[0].view(), point[0])
        };
        let x_u = x_l + 1;
        let x_diff = (point[0] - data.grid[0][x_l]) / (data.grid[0][x_u] - data.grid[0][x_l]);
        Ok(data.values[x_l] * (1.0 - x_diff) + data.values[x_u] * x_diff)
    }

    /// Returns `true`
    fn allow_extrapolate(&self) -> bool {
        true
    }
}

impl Strategy1D for Nearest {
    fn interpolate(&self, data: &InterpData1D, point: &[f64; 1]) -> Result<f64, InterpolateError> {
        if let Some(i) = data.grid[0].iter().position(|&x_val| x_val == point[0]) {
            return Ok(data.values[i]);
        }
        let x_l = find_nearest_index(data.grid[0].view(), point[0]);
        let x_u = x_l + 1;
        let x_diff = (point[0] - data.grid[0][x_l]) / (data.grid[0][x_u] - data.grid[0][x_l]);
        let i = if x_diff < 0.5 { x_l } else { x_u };
        Ok(data.values[i])
    }

    /// Returns `false`
    fn allow_extrapolate(&self) -> bool {
        false
    }
}

impl Strategy1D for LeftNearest {
    fn interpolate(&self, data: &InterpData1D, point: &[f64; 1]) -> Result<f64, InterpolateError> {
        if let Some(i) = data.grid[0].iter().position(|&x_val| x_val == point[0]) {
            return Ok(data.values[i]);
        }
        let x_l = find_nearest_index(data.grid[0].view(), point[0]);
        Ok(data.values[x_l])
    }

    /// Returns `false`
    fn allow_extrapolate(&self) -> bool {
        false
    }
}

impl Strategy1D for RightNearest {
    fn interpolate(&self, data: &InterpData1D, point: &[f64; 1]) -> Result<f64, InterpolateError> {
        if let Some(i) = data.grid[0].iter().position(|&x_val| x_val == point[0]) {
            return Ok(data.values[i]);
        }
        let x_u = find_nearest_index(data.grid[0].view(), point[0]) + 1;
        Ok(data.values[x_u])
    }

    /// Returns `false`
    fn allow_extrapolate(&self) -> bool {
        false
    }
}
