use super::*;

#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Cubic<T: Default> {
    pub boundary_cond: CubicBC<T>,
    pub z: Array1<T>,
}

impl<T> Cubic<T>
where
    T: Default,
{
    pub fn new(bc: CubicBC<T>) -> Self {
        Self {
            boundary_cond: bc,
            z: <Array1<T> as Default>::default(),
        }
    }

    pub fn natural() -> Self {
        Self::new(CubicBC::Natural)
    }

    pub fn clamped(a: T, b: T) -> Self {
        Self::new(CubicBC::Clamped(a, b))
    }

    pub fn not_a_knot() -> Self {
        Self::new(CubicBC::NotAKnot)
    }

    pub fn periodic() -> Self {
        Self::new(CubicBC::Periodic)
    }
}

impl<T> Cubic<T>
where
    T: Num + Copy + Default,
{
    /// Solves Ax = d for a tridiagonal matrix A using the [Thomas algorithm](https://en.wikipedia.org/wiki/Tridiagonal_matrix_algorithm).
    /// - `a`: sub-diagonal (1 element shorter than `b` and `d`)
    /// - `b`: diagonal
    /// - `c`: super-diagonal (1 element shorter than `b` and `d`)
    /// - `d`: right-hand side
    pub fn thomas(
        a: ArrayView1<T>,
        b: ArrayView1<T>,
        c: ArrayView1<T>,
        d: ArrayView1<T>,
    ) -> Array1<T> {
        let n = d.len();
        assert_eq!(a.len(), n - 1);
        assert_eq!(b.len(), n);
        assert_eq!(c.len(), n - 1);

        let mut c_prime = Array1::default(n - 1);
        let mut d_prime = Array1::default(n);
        let mut x = Array1::default(n);

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

/// Cubic spline boundary conditions.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum CubicBC<T> {
    /// Second derivatives at endpoints are 0, thus extrapolation is linear.
    // https://www.math.ntnu.no/emner/TMA4215/2008h/cubicsplines.pdf
    #[default]
    Natural,
    /// Specific first derivatives at endpoints.
    Clamped(T, T),
    NotAKnot,
    // https://math.ou.edu/~npetrov/project-5093-s11.pdf
    Periodic,
}
