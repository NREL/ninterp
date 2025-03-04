use super::*;

impl Interp3DStrategy for Linear {
    fn interpolate(&self, interp: &Interp3D, point: &[f64]) -> Result<f64, InterpolateError> {
        // Extrapolation is checked previously in Interpolator::interpolate,
        // meaning:
        // - point is within grid bounds, or
        // - point is clamped, or
        // - extrapolation is enabled
        let grid = [&interp.x, &interp.y, &interp.z];
        let lowers: Vec<usize> = (0..3)
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
        let x_diff = (point[0] - interp.x[x_l]) / (interp.x[x_u] - interp.x[x_l]);
        // y
        let y_l = lowers[1];
        let y_u = y_l + 1;
        let y_diff = (point[1] - interp.y[y_l]) / (interp.y[y_u] - interp.y[y_l]);
        // z
        let z_l = lowers[2];
        let z_u = z_l + 1;
        let z_diff = (point[2] - interp.z[z_l]) / (interp.z[z_u] - interp.z[z_l]);
        // interpolate in the x-direction
        let f00 = interp.f_xyz[x_l][y_l][z_l] * (1.0 - x_diff) + interp.f_xyz[x_u][y_l][z_l] * x_diff;
        let f01 = interp.f_xyz[x_l][y_l][z_u] * (1.0 - x_diff) + interp.f_xyz[x_u][y_l][z_u] * x_diff;
        let f10 = interp.f_xyz[x_l][y_u][z_l] * (1.0 - x_diff) + interp.f_xyz[x_u][y_u][z_l] * x_diff;
        let f11 = interp.f_xyz[x_l][y_u][z_u] * (1.0 - x_diff) + interp.f_xyz[x_u][y_u][z_u] * x_diff;
        // interpolate in the y-direction
        let f0 = f00 * (1.0 - y_diff) + f10 * y_diff;
        let f1 = f01 * (1.0 - y_diff) + f11 * y_diff;
        // interpolate in the z-direction
        Ok(f0 * (1.0 - z_diff) + f1 * z_diff)
    }

    fn allow_extrapolate(&self) -> bool {
        true
    }
}

impl Interp3DStrategy for Nearest {
    fn interpolate(&self, interp: &Interp3D, point: &[f64]) -> Result<f64, InterpolateError> {
        // x
        let x_l = find_nearest_index(&interp.x, point[0]);
        let x_u = x_l + 1;
        let x_diff = (point[0] - interp.x[x_l]) / (interp.x[x_u] - interp.x[x_l]);
        let i = if x_diff < 0.5 { x_l } else { x_u };
        // y
        let y_l = find_nearest_index(&interp.y, point[1]);
        let y_u = y_l + 1;
        let y_diff = (point[1] - interp.y[y_l]) / (interp.y[y_u] - interp.y[y_l]);
        let j = if y_diff < 0.5 { y_l } else { y_u };
        // z
        let z_l = find_nearest_index(&interp.z, point[2]);
        let z_u = z_l + 1;
        let z_diff = (point[2] - interp.z[z_l]) / (interp.z[z_u] - interp.z[z_l]);
        let k = if z_diff < 0.5 { z_l } else { z_u };

        Ok(interp.f_xyz[i][j][k])
    }

    fn allow_extrapolate(&self) -> bool {
        false
    }
}