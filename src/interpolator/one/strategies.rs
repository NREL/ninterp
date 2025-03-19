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
    D::Elem: Float + Euclid + Debug,
{
    fn init(&mut self, data: &InterpData1D<D>) -> Result<(), ValidateError> {
        // Number of segments
        let n = data.grid[0].len() - 1;

        let zero = D::Elem::zero();
        let one = D::Elem::one();
        let two = <D::Elem as NumCast>::from(2.).unwrap();
        let six = <D::Elem as NumCast>::from(6.).unwrap();

        let h = Array1::from_shape_fn(n, |i| data.grid[0][i + 1] - data.grid[0][i]);
        let v = Array1::from_shape_fn(n - 1, |i| two * (h[i + 1] + h[i]));
        let b = Array1::from_shape_fn(n, |i| (data.values[i + 1] - data.values[i]) / h[i]);
        let u = Array1::from_shape_fn(n - 1, |i| six * (b[i + 1] - b[i]));

        let (sub, diag, sup, rhs) = match &self.boundary_condition {
            CubicBC::Natural => {
                let zero = array![zero];
                let one = array![one];
                (
                    &ndarray::concatenate(Axis(0), &[h.slice(s![0..n - 1]), zero.view()]).unwrap(),
                    &ndarray::concatenate(Axis(0), &[one.view(), v.view(), one.view()]).unwrap(),
                    &ndarray::concatenate(Axis(0), &[zero.view(), h.slice(s![1..n])]).unwrap(),
                    &ndarray::concatenate(Axis(0), &[zero.view(), u.view(), zero.view()]).unwrap(),
                )
            }
            CubicBC::Clamped(l, r) => {
                let diag_0 = array![two * h[0]];
                let diag_n = array![two * h[n - 1]];
                let rhs_0 = array![six * (b[0] - *l)];
                let rhs_n = array![six * (*r - b[n - 1])];
                (
                    &h,
                    &ndarray::concatenate(Axis(0), &[diag_0.view(), v.view(), diag_n.view()])
                        .unwrap(),
                    &h,
                    &ndarray::concatenate(Axis(0), &[rhs_0.view(), u.view(), rhs_n.view()])
                        .unwrap(),
                )
            }
            CubicBC::NotAKnot => {
                let three = two + one;
                let sub_n =
                    array![two * h[n - 1].powi(2) + three * h[n - 1] * h[n - 2] + h[n - 2].powi(2)];
                let diag_0 = array![h[0].powi(2) - h[1].powi(2)];
                let diag_n = array![h[n - 1].powi(2) - h[n - 2].powi(2)];
                let sup_0 = array![two * h[0].powi(2) + three * h[0] * h[1] + h[1].powi(2)];
                let rhs_0 = array![h[0] * u[0]];
                let rhs_n = array![h[n - 1] * u[n - 2]];

                println!(
                    "sub {:?}",
                    &ndarray::concatenate(Axis(0), &[h.slice(s![0..n - 1]), sub_n.view()]).unwrap()
                );
                println!(
                    "dia {:?}",
                    &ndarray::concatenate(Axis(0), &[diag_0.view(), v.view(), diag_n.view()])
                        .unwrap()
                );
                println!(
                    "sup {:?}",
                    &ndarray::concatenate(Axis(0), &[sup_0.view(), h.slice(s![1..n])]).unwrap()
                );
                println!(
                    "rhs {:?}",
                    &ndarray::concatenate(Axis(0), &[rhs_0.view(), u.view(), rhs_n.view()])
                        .unwrap()
                );
                (
                    &ndarray::concatenate(Axis(0), &[h.slice(s![0..n - 1]), sub_n.view()]).unwrap(),
                    &ndarray::concatenate(Axis(0), &[diag_0.view(), v.view(), diag_n.view()])
                        .unwrap(),
                    &ndarray::concatenate(Axis(0), &[sup_0.view(), h.slice(s![1..n])]).unwrap(),
                    &ndarray::concatenate(Axis(0), &[rhs_0.view(), u.view(), rhs_n.view()])
                        .unwrap(),
                )
            }
            _ => unreachable!(),
        };

        self.z = Self::thomas(sub.view(), diag.view(), sup.view(), rhs.view()).into_dyn();

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
