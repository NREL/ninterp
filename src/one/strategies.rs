use super::*;

impl<D> Strategy1D<D> for Linear
where
    D: Data + RawDataClone + Clone,
    D::Elem: Num + PartialOrd + Copy + Debug,
{
    fn interpolate(
        &self,
        data: &InterpData1D<D>,
        point: &[D::Elem; 1],
    ) -> Result<D::Elem, InterpolateError> {
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
            find_nearest_index(data.grid[0].view(), &point[0])
        };
        let x_u = x_l + 1;
        let x_diff = (point[0] - data.grid[0][x_l]) / (data.grid[0][x_u] - data.grid[0][x_l]);
        Ok(data.values[x_l] * (D::Elem::one() - x_diff) + data.values[x_u] * x_diff)
    }

    /// Returns `true`.
    fn allow_extrapolate(&self) -> bool {
        true
    }
}

impl<D> Strategy1D<D> for Cubic<D::Elem>
where
    D: Data + RawDataClone + Clone,
    D::Elem: Float + Euclid + Default + Debug,
{
    fn init(&mut self, data: &InterpData1D<D>) -> Result<(), ValidateError> {
        // Number of segments
        let n = data.grid[0].len() - 1;

        let zero = D::Elem::zero();
        let one = D::Elem::one();
        let two = <D::Elem as NumCast>::from(2.).unwrap();
        let six = <D::Elem as NumCast>::from(6.).unwrap();

        let h = Array1::from_shape_fn(n, |i| data.grid[0][i + 1] - data.grid[0][i]);
        let v = Array1::from_shape_fn(n + 1, |i| {
            if i == 0 || i == n {
                match &self.boundary_condition {
                    CubicBC::Natural => one,
                    CubicBC::Clamped(_, _) => two * h[0],
                    _ => todo!(),
                }
            } else {
                two * (h[i - 1] + h[i])
            }
        });
        let b = Array1::from_shape_fn(n, |i| (data.values[i + 1] - data.values[i]) / h[i]);
        let u = Array1::from_shape_fn(n + 1, |i| {
            if i == 0 || i == n {
                match &self.boundary_condition {
                    CubicBC::Natural => zero,
                    CubicBC::Clamped(l, r) => {
                        if i == 0 {
                            six * (b[i] - *l)
                        } else {
                            six * (*r - b[i - 1])
                        }
                    }
                    _ => todo!(),
                }
            } else {
                six * (b[i] - b[i - 1])
            }
        });

        let (sub, sup) = match &self.boundary_condition {
            CubicBC::Natural => (
                &Array1::from_shape_fn(n, |i| if i == n - 1 { zero } else { h[i] }),
                &Array1::from_shape_fn(n, |i| if i == 0 { zero } else { h[i] }),
            ),
            CubicBC::Clamped(_, _) => (&h, &h),
            _ => todo!(),
        };

        self.z = Self::thomas(sub.view(), v.view(), sup.view(), u.view());

        Ok(())
    }

    fn interpolate(
        &self,
        data: &InterpData1D<D>,
        point: &[<D>::Elem; 1],
    ) -> Result<<D>::Elem, InterpolateError> {
        let last = data.grid[0].len() - 1;
        let l = if point[0] < data.grid[0][0] {
            match &self.extrapolate {
                CubicExtrapolate::Linear => {
                    let h0 = data.grid[0][1] - data.grid[0][0];
                    let k0 = (data.values[1] - data.values[0]) / h0
                        - h0 * self.z[1] / <D::Elem as NumCast>::from(6.).unwrap();
                    return Ok(k0 * (point[0] - data.grid[0][0]) + data.values[0]);
                }
                CubicExtrapolate::Spline => 0,
                CubicExtrapolate::Wrap => {
                    let point = [wrap(point[0], data.grid[0][0], data.grid[0][last])];
                    let l = find_nearest_index(data.grid[0].view(), &point[0]);
                    return self.evaluate_1d(&point, l, data);
                }
            }
        } else if point[0] > data.grid[0][last] {
            match &self.extrapolate {
                CubicExtrapolate::Linear => {
                    let hn = data.grid[0][last] - data.grid[0][last - 1];
                    let kn = (data.values[last] - data.values[last - 1]) / hn
                        + hn * self.z[last - 1] / <D::Elem as NumCast>::from(6.).unwrap();
                    return Ok(kn * (point[0] - data.grid[0][last]) + data.values[last]);
                }
                CubicExtrapolate::Spline => last - 1,
                CubicExtrapolate::Wrap => {
                    let point = [wrap(point[0], data.grid[0][0], data.grid[0][last])];
                    let l = find_nearest_index(data.grid[0].view(), &point[0]);
                    return self.evaluate_1d(&point, l, data);
                }
            }
        } else {
            find_nearest_index(data.grid[0].view(), &point[0])
        };
        self.evaluate_1d(point, l, data)
    }

    /// Returns `true`
    fn allow_extrapolate(&self) -> bool {
        true
    }
}

impl<D> Strategy1D<D> for Nearest
where
    D: Data + RawDataClone + Clone,
    D::Elem: Num + PartialOrd + Copy + Debug,
{
    fn interpolate(
        &self,
        data: &InterpData1D<D>,
        point: &[D::Elem; 1],
    ) -> Result<D::Elem, InterpolateError> {
        if let Some(i) = data.grid[0].iter().position(|&x_val| x_val == point[0]) {
            return Ok(data.values[i]);
        }
        let x_l = find_nearest_index(data.grid[0].view(), &point[0]);
        let x_u = x_l + 1;
        let i = if point[0] - data.grid[0][x_l] < data.grid[0][x_u] - point[0] {
            x_l
        } else {
            x_u
        };
        Ok(data.values[i])
    }

    /// Returns `false`.
    fn allow_extrapolate(&self) -> bool {
        false
    }
}

impl<D> Strategy1D<D> for LeftNearest
where
    D: Data + RawDataClone + Clone,
    D::Elem: Num + PartialOrd + Copy + Debug,
{
    fn interpolate(
        &self,
        data: &InterpData1D<D>,
        point: &[D::Elem; 1],
    ) -> Result<D::Elem, InterpolateError> {
        if let Some(i) = data.grid[0].iter().position(|&x_val| x_val == point[0]) {
            return Ok(data.values[i]);
        }
        let x_l = find_nearest_index(data.grid[0].view(), &point[0]);
        Ok(data.values[x_l])
    }

    /// Returns `false`.
    fn allow_extrapolate(&self) -> bool {
        false
    }
}

impl<D> Strategy1D<D> for RightNearest
where
    D: Data + RawDataClone + Clone,
    D::Elem: Num + PartialOrd + Copy + Debug,
{
    fn interpolate(
        &self,
        data: &InterpData1D<D>,
        point: &[D::Elem; 1],
    ) -> Result<D::Elem, InterpolateError> {
        if let Some(i) = data.grid[0].iter().position(|&x_val| x_val == point[0]) {
            return Ok(data.values[i]);
        }
        let x_u = find_nearest_index(data.grid[0].view(), &point[0]) + 1;
        Ok(data.values[x_u])
    }

    /// Returns `false`.
    fn allow_extrapolate(&self) -> bool {
        false
    }
}
