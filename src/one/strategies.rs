use super::*;

impl Strategy1D for Linear {
    fn interpolate(&self, interpolator: &Interp1D, point: &[f64; 1]) -> Result<f64, InterpolateError> {
        if let Some(i) = interpolator.x.iter().position(|&x_val| x_val == point[0]) {
            return Ok(interpolator.f_x[i]);
        }
        // Extrapolation is checked previously in `Interpolator::interpolate`,
        // meaning:
        // - point is within grid bounds, or
        // - point is clamped, or
        // - extrapolation is enabled
        let x_l = if &point[0] < interpolator.x.first().unwrap() {
            0
        } else if &point[0] > interpolator.x.last().unwrap() {
            interpolator.x.len() - 2
        } else {
            find_nearest_index(&interpolator.x, point[0])
        };
        let x_u = x_l + 1;
        let x_diff = (point[0] - interpolator.x[x_l]) / (interpolator.x[x_u] - interpolator.x[x_l]);
        Ok(interpolator.f_x[x_l] * (1.0 - x_diff) + interpolator.f_x[x_u] * x_diff)
    }

    fn allow_extrapolate(&self) -> bool {
        true
    }
}

impl Strategy1D for Nearest {
    fn interpolate(&self, interpolator: &Interp1D, point: &[f64; 1]) -> Result<f64, InterpolateError> {
        if let Some(i) = interpolator.x.iter().position(|&x_val| x_val == point[0]) {
            return Ok(interpolator.f_x[i]);
        }
        let x_l = find_nearest_index(&interpolator.x, point[0]);
        let x_u = x_l + 1;
        let x_diff = (point[0] - interpolator.x[x_l]) / (interpolator.x[x_u] - interpolator.x[x_l]);
        let i = if x_diff < 0.5 { x_l } else { x_u };
        Ok(interpolator.f_x[i])
    }

    fn allow_extrapolate(&self) -> bool {
        false
    }
}

impl Strategy1D for LeftNearest {
    fn interpolate(&self, interpolator: &Interp1D, point: &[f64; 1]) -> Result<f64, InterpolateError> {
        if let Some(i) = interpolator.x.iter().position(|&x_val| x_val == point[0]) {
            return Ok(interpolator.f_x[i]);
        }
        let x_l = find_nearest_index(&interpolator.x, point[0]);
        Ok(interpolator.f_x[x_l])
    }

    fn allow_extrapolate(&self) -> bool {
        false
    }
}

impl Strategy1D for RightNearest {
    fn interpolate(&self, interpolator: &Interp1D, point: &[f64; 1]) -> Result<f64, InterpolateError> {
        if let Some(i) = interpolator.x.iter().position(|&x_val| x_val == point[0]) {
            return Ok(interpolator.f_x[i]);
        }
        let x_u = find_nearest_index(&interpolator.x, point[0]) + 1;
        Ok(interpolator.f_x[x_u])
    }

    fn allow_extrapolate(&self) -> bool {
        false
    }
}
