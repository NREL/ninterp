use super::*;

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Cubic<T> {
    /// Cubic spline boundary conditions.
    pub boundary_condition: CubicBC<T>,
    /// Behavior of [`Extrapolate::Enable`].
    pub extrapolate: CubicExtrapolate,
    /// Solved second derivatives.
    pub z: ArrayD<T>,
}

/// Cubic spline boundary conditions.
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum CubicBC<T> {
    Natural,
    Clamped(T, T),
    NotAKnot,
    // https://math.ou.edu/~npetrov/project-5093-s11.pdf
    Periodic,
}

/// [`Extrapolate::Enable`] behavior for cubic splines
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum CubicExtrapolate {
    /// Linear extrapolation, default for natural splines.
    Linear,
    /// Use nearest spline to extrapolate.
    Spline,
    /// Same as [`Extrapolate::Wrap`], default for periodic splines.
    Wrap,
}

impl<T> Cubic<T> {
    /// Cubic spline with given boundary condition and extrapolation behavior.
    pub fn new(boundary_condition: CubicBC<T>, extrapolate: CubicExtrapolate) -> Self {
        Self {
            boundary_condition,
            extrapolate,
            z: Array1::from_vec(Vec::new()).into_dyn(),
        }
    }

    /// Natural cubic spline
    /// (splines straighten at outermost knots).
    ///
    /// 2nd derivatives at outermost knots are zero:
    /// z<sub>0</sub> = z<sub>n</sub> = 0
    ///
    /// [`Extrapolate::Enable`] defaults to [`CubicExtrapolate::Linear`].
    pub fn natural() -> Self {
        Self::new(CubicBC::Natural, CubicExtrapolate::Linear)
    }

    /// Clamped cubic spline.
    ///
    /// 1st derivatives at outermost knots (k<sub>0</sub>, k<sub>n</sub>) are given.
    ///
    /// [`Extrapolate::Enable`] defaults to [`CubicExtrapolate::Spline`].
    pub fn clamped(k0: T, kn: T) -> Self {
        Self::new(CubicBC::Clamped(k0, kn), CubicExtrapolate::Spline)
    }

    /// Not-a-knot cubic spline.
    ///
    /// Spline 3rd derivatives at second and second-to-last knots are continuous, respectively:
    /// S'''<sub>0</sub>(x<sub>1</sub>) = S'''<sub>1</sub>(x<sub>1</sub>) and
    /// S'''<sub>n-1</sub>(x<sub>n-1</sub>) = S'''<sub>n</sub>(x<sub>n-1</sub>).
    ///
    /// In other words, this means the first and second spline at data boundaries are the same cubic.
    ///
    /// [`Extrapolate::Enable`] defaults to [`CubicExtrapolate::Spline`].
    pub fn not_a_knot() -> Self {
        Self::new(CubicBC::NotAKnot, CubicExtrapolate::Spline)
    }

    /// Periodic cubic spline.
    ///
    /// Spline 1st & 2nd derivatives at outermost knots are equal:
    /// k<sub>0</sub> = k<sub>n</sub>, z<sub>0</sub> = z<sub>n</sub>
    ///
    /// [`Extrapolate::Enable`] defaults to [`CubicExtrapolate::Wrap`],
    /// thus is equivalent to [`Extrapolate::Wrap`].
    pub fn periodic() -> Self {
        Self::new(CubicBC::Periodic, CubicExtrapolate::Wrap)
    }
}

impl<T> Cubic<T>
where
    T: Float + Debug,
{
    // Reference: https://www.math.ntnu.no/emner/TMA4215/2008h/cubicsplines.pdf
    pub(crate) fn evaluate_1d<D: Data<Elem = T> + RawDataClone + Clone>(
        &self,
        point: &[T; 1],
        l: usize,
        data: &InterpData1D<D>,
    ) -> Result<T, InterpolateError> {
        let six = <D::Elem as NumCast>::from(6.).unwrap();
        let u = l + 1;
        let h_i = data.grid[0][u] - data.grid[0][l];
        Ok(
            self.z[u] / (six * h_i) * (point[0] - data.grid[0][l]).powi(3)
                + self.z[l] / (six * h_i) * (data.grid[0][u] - point[0]).powi(3)
                + (data.values[u] / h_i - self.z[u] * h_i / six) * (point[0] - data.grid[0][l])
                + (data.values[l] / h_i - self.z[l] * h_i / six) * (data.grid[0][u] - point[0]),
        )
    }

    pub(crate) fn evaluate_2d<D: Data<Elem = T> + RawDataClone + Clone>(
        &self,
        point: &[T; 2],
        l: usize,
        data: &InterpData2D<D>,
    ) -> Result<T, InterpolateError> {
        todo!()
    }

    pub(crate) fn evaluate_3d<D: Data<Elem = T> + RawDataClone + Clone>(
        &self,
        point: &[T; 3],
        l: usize,
        data: &InterpData3D<D>,
    ) -> Result<T, InterpolateError> {
        todo!()
    }

    pub(crate) fn evaluate_nd<D: Data<Elem = T> + RawDataClone + Clone>(
        &self,
        point: &[T],
        l: usize,
        data: &InterpDataND<D>,
    ) -> Result<T, InterpolateError> {
        todo!()
    }
}

impl<T> Cubic<T>
where
    T: Num + Copy,
{
    /// Solves Ax = d for a tridiagonal matrix A using the
    /// [Thomas algorithm](https://en.wikipedia.org/wiki/Tridiagonal_matrix_algorithm).
    /// - `a`: sub-diagonal (1 element shorter than `b` and `d`)
    /// - `b`: diagonal
    /// - `c`: super-diagonal (1 element shorter than `b` and `d`)
    /// - `d`: right-hand side
    pub(crate) fn thomas(
        a: ArrayView1<T>,
        b: ArrayView1<T>,
        c: ArrayView1<T>,
        d: ArrayView1<T>,
    ) -> Array1<T> {
        let n = d.len();
        assert_eq!(a.len(), n - 1);
        assert_eq!(b.len(), n);
        assert_eq!(c.len(), n - 1);

        let mut c_prime = Array1::zeros(n - 1);
        let mut d_prime = Array1::zeros(n);
        let mut x = Array1::zeros(n);

        // Forward sweep
        c_prime[0] = c[0] / b[0];
        d_prime[0] = d[0] / b[0];

        for i in 1..n - 1 {
            let denom = b[i] - a[i - 1] * c_prime[i - 1];
            c_prime[i] = c[i] / denom;
            d_prime[i] = (d[i] - a[i - 1] * d_prime[i - 1]) / denom;
        }
        d_prime[n - 1] =
            (d[n - 1] - a[n - 2] * d_prime[n - 2]) / (b[n - 1] - a[n - 2] * c_prime[n - 2]);

        // Back substitution
        x[n - 1] = d_prime[n - 1];
        for i in (0..n - 1).rev() {
            x[i] = d_prime[i] - c_prime[i] * x[i + 1];
        }

        x
    }
}
