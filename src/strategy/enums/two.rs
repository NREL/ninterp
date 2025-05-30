use super::*;

/// See [enums module](super) documentation.
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum Strategy2DEnum {
    Linear(strategy::Linear),
    Nearest(strategy::Nearest),
}

impl From<Linear> for Strategy2DEnum {
    #[inline]
    fn from(strategy: Linear) -> Self {
        Self::Linear(strategy)
    }
}

impl From<Nearest> for Strategy2DEnum {
    #[inline]
    fn from(strategy: Nearest) -> Self {
        Self::Nearest(strategy)
    }
}

impl<D> Strategy2D<D> for Strategy2DEnum
where
    D: Data + RawDataClone + Clone,
    D::Elem: Num + PartialOrd + Copy + Debug,
{
    #[inline]
    fn init(&mut self, data: &InterpData2D<D>) -> Result<(), ValidateError> {
        match self {
            Strategy2DEnum::Linear(strategy) => Strategy2D::<D>::init(strategy, data),
            Strategy2DEnum::Nearest(strategy) => Strategy2D::<D>::init(strategy, data),
        }
    }

    #[inline]
    fn interpolate(
        &self,
        data: &InterpData2D<D>,
        point: &[D::Elem; 2],
    ) -> Result<D::Elem, InterpolateError> {
        match self {
            Strategy2DEnum::Linear(strategy) => Strategy2D::<D>::interpolate(strategy, data, point),
            Strategy2DEnum::Nearest(strategy) => {
                Strategy2D::<D>::interpolate(strategy, data, point)
            }
        }
    }

    #[inline]
    fn allow_extrapolate(&self) -> bool {
        match self {
            Strategy2DEnum::Linear(strategy) => Strategy2D::<D>::allow_extrapolate(strategy),
            Strategy2DEnum::Nearest(strategy) => Strategy2D::<D>::allow_extrapolate(strategy),
        }
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    #[cfg(feature = "serde")]
    fn test_serde() {
        assert_eq!(
            serde_json::to_string(&Strategy2DEnum::from(Linear)).unwrap(),
            serde_json::to_string(&Linear).unwrap(),
        );
        assert_eq!(
            serde_json::to_string(&Strategy2DEnum::from(Nearest)).unwrap(),
            serde_json::to_string(&Nearest).unwrap(),
        );
    }
}
