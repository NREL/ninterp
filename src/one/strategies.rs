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
    D::Elem: Float + Default + Debug,
{
    /// Solves coefficients

    /// Reference: https://www.math.ntnu.no/emner/TMA4215/2008h/cubicsplines.pdf
    /// ```text
    /// ┌─                          ─┐┌─    ─┐    ┌─    ─┐  
    /// │ V1 H1                      ││  Z1  │    │  U1  │  
    /// │ H1 V2 H2                   ││  Z2  │    │  U2  │  
    /// │    H2 V3 H3                ││  Z3  │    │  U3  │  
    /// │       .  .  .              ││   .  │ ── │   .  │  
    /// │          .  .  .           ││   .  │ ── │   .  │  
    /// │             .  .  .        ││   .  │    │   .  │  
    /// │                . Vn-2 Hn-2 ││ Zn-2 │    │ Un-2 │  
    /// │                  Hn-2 Vn-1 ││ Zn-1 │    │ Un-1 │  
    /// └─                          ─┘└─    ─┘    └─    ─┘  
    /// ```
    fn init(&mut self, data: &InterpData1D<D>) -> Result<(), ValidateError> {
        // Number of segments
        let n = data.grid[0].len() - 1;

        let zero = D::Elem::zero();
        let one = D::Elem::one();
        let two = <D::Elem as NumCast>::from(2.).unwrap();
        let six = <D::Elem as NumCast>::from(6.).unwrap();

        // let h = Array1::from_shape_fn(n, |i| data.grid[0][i + 1] - data.grid[0][i]);
        // let v = Array1::from_shape_fn(n - 1, |i| two * (h[i] + h[i + 1]));
        // let b = Array1::from_shape_fn(n, |i| (data.values[i + 1] - data.values[i]) / h[i]);
        // let u = Array1::from_shape_fn(n - 1, |i| six * (b[i + 1] - b[i]));

        let h = Array1::from_shape_fn(n, |i| data.grid[0][i + 1] - data.grid[0][i]);
        let v = Array1::from_shape_fn(n + 1, |i| {
            if i == 0 || i == n {
                match &self.boundary_cond {
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
                match &self.boundary_cond {
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

        let (sub, sup) = match &self.boundary_cond {
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
        let l = if &point[0] < data.grid[0].first().unwrap() {
            0
        } else if &point[0] > data.grid[0].last().unwrap() {
            data.grid[0].len() - 2
        } else {
            find_nearest_index(data.grid[0].view(), &point[0])
        };
        let u = l + 1;

        let six = <D::Elem as NumCast>::from(6.).unwrap();
        let h_i = data.grid[0][u] - data.grid[0][l];

        Ok(
            self.z[u] / (six * h_i) * (point[0] - data.grid[0][l]).powi(3)
                + self.z[l] / (six * h_i) * (data.grid[0][u] - point[0]).powi(3)
                + (data.values[u] / h_i - self.z[u] * h_i / six) * (point[0] - data.grid[0][l])
                + (data.values[l] / h_i - self.z[l] * h_i / six) * (data.grid[0][u] - point[0]),
        )
    }

    /// Returns `true`
    fn allow_extrapolate(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::*;

    #[test]
    fn test_cubic_natural() {
        let x = array![1., 2., 3., 5., 7., 8.];
        let f_x = array![3., 6., 19., 99., 291., 444.];

        let interp =
            Interp1D::new(x.view(), f_x.view(), Cubic::natural(), Extrapolate::Enable).unwrap();

        for i in 0..x.len() {
            assert_approx_eq!(interp.interpolate(&[x[i]]).unwrap(), f_x[i])
        }
    }

    #[test]
    fn test_cubic_clamped() {
        let x = array![1., 2., 3., 5., 7., 8.];
        let f_x = array![3., 6., 19., 99., 291., 444.];

        let interp = Interp1D::new(
            x.view(),
            f_x.view(),
            Cubic::clamped(0., 0.),
            Extrapolate::Enable,
        )
        .unwrap();

        for i in 0..x.len() {
            assert_approx_eq!(interp.interpolate(&[x[i]]).unwrap(), f_x[i])
        }
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
