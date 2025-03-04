use super::*;

impl Interp1DStrategy for Linear {
    fn interpolate(&self, interp: &Interp1D, point: &[f64]) -> Result<f64, InterpolateError> {
        if let Some(i) = interp.x.iter().position(|&x_val| x_val == point[0]) {
            return Ok(interp.f_x[i]);
        }
        // Extrapolation is checked previously in `Interpolator::interpolate`,
        // meaning:
        // - point is within grid bounds, or
        // - point is clamped, or
        // - extrapolation is enabled
        let x_l = if &point[0] < interp.x.first().unwrap() {
            0
        } else if &point[0] > interp.x.last().unwrap() {
            interp.x.len() - 2
        } else {
            find_nearest_index(&interp.x, point[0])
        };
        let x_u = x_l + 1;
        let x_diff = (point[0] - interp.x[x_l]) / (interp.x[x_u] - interp.x[x_l]);
        Ok(interp.f_x[x_l] * (1.0 - x_diff) + interp.f_x[x_u] * x_diff)
    }

    fn allow_extrapolate(&self) -> bool {
        true
    }
}

impl Interp1DStrategy for Nearest {
    fn interpolate(&self, interp: &Interp1D, point: &[f64]) -> Result<f64, InterpolateError> {
        if let Some(i) = interp.x.iter().position(|&x_val| x_val == point[0]) {
            return Ok(interp.f_x[i]);
        }
        let x_l = find_nearest_index(&interp.x, point[0]);
        let x_u = x_l + 1;
        let x_diff = (point[0] - interp.x[x_l]) / (interp.x[x_u] - interp.x[x_l]);
        let i = if x_diff < 0.5 { x_l } else { x_u };
        Ok(interp.f_x[i])
    }

    fn allow_extrapolate(&self) -> bool {
        false
    }
}

impl Interp1DStrategy for LeftNearest {
    fn interpolate(&self, interp: &Interp1D, point: &[f64]) -> Result<f64, InterpolateError> {
        if let Some(i) = interp.x.iter().position(|&x_val| x_val == point[0]) {
            return Ok(interp.f_x[i]);
        }
        let x_l = find_nearest_index(&interp.x, point[0]);
        Ok(interp.f_x[x_l])
    }

    fn allow_extrapolate(&self) -> bool {
        false
    }
}

impl Interp1DStrategy for RightNearest {
    fn interpolate(&self, interp: &Interp1D, point: &[f64]) -> Result<f64, InterpolateError> {
        if let Some(i) = interp.x.iter().position(|&x_val| x_val == point[0]) {
            return Ok(interp.f_x[i]);
        }
        let x_u = find_nearest_index(&interp.x, point[0]) + 1;
        Ok(interp.f_x[x_u])
    }

    fn allow_extrapolate(&self) -> bool {
        false
    }
}