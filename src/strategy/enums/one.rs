use super::*;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum Strategy1DEnum {
    Linear(strategy::Linear),
    Nearest(strategy::Nearest),
    LeftNearest(strategy::LeftNearest),
    RightNearest(strategy::RightNearest),
}

impl From<Linear> for Strategy1DEnum {
    #[inline]
    fn from(strategy: Linear) -> Self {
        Self::Linear(strategy)
    }
}

impl From<Nearest> for Strategy1DEnum {
    #[inline]
    fn from(strategy: Nearest) -> Self {
        Self::Nearest(strategy)
    }
}

impl From<LeftNearest> for Strategy1DEnum {
    #[inline]
    fn from(strategy: LeftNearest) -> Self {
        Self::LeftNearest(strategy)
    }
}

impl From<RightNearest> for Strategy1DEnum {
    #[inline]
    fn from(strategy: RightNearest) -> Self {
        Self::RightNearest(strategy)
    }
}

impl<D> Strategy1D<D> for Strategy1DEnum
where
    D: Data + RawDataClone + Clone,
    D::Elem: Num + PartialOrd + Copy + Debug,
{
    #[inline]
    fn init(&mut self, data: &InterpData1D<D>) -> Result<(), ValidateError> {
        match self {
            Strategy1DEnum::Linear(strategy) => Strategy1D::<D>::init(strategy, data),
            Strategy1DEnum::Nearest(strategy) => Strategy1D::<D>::init(strategy, data),
            Strategy1DEnum::LeftNearest(strategy) => Strategy1D::<D>::init(strategy, data),
            Strategy1DEnum::RightNearest(strategy) => Strategy1D::<D>::init(strategy, data),
        }
    }

    #[inline]
    fn interpolate(
        &self,
        data: &InterpData1D<D>,
        point: &[D::Elem; 1],
    ) -> Result<D::Elem, InterpolateError> {
        match self {
            Strategy1DEnum::Linear(strategy) => Strategy1D::<D>::interpolate(strategy, data, point),
            Strategy1DEnum::Nearest(strategy) => {
                Strategy1D::<D>::interpolate(strategy, data, point)
            }
            Strategy1DEnum::LeftNearest(strategy) => {
                Strategy1D::<D>::interpolate(strategy, data, point)
            }
            Strategy1DEnum::RightNearest(strategy) => {
                Strategy1D::<D>::interpolate(strategy, data, point)
            }
        }
    }

    #[inline]
    fn allow_extrapolate(&self) -> bool {
        match self {
            Strategy1DEnum::Linear(strategy) => Strategy1D::<D>::allow_extrapolate(strategy),
            Strategy1DEnum::Nearest(strategy) => Strategy1D::<D>::allow_extrapolate(strategy),
            Strategy1DEnum::LeftNearest(strategy) => Strategy1D::<D>::allow_extrapolate(strategy),
            Strategy1DEnum::RightNearest(strategy) => Strategy1D::<D>::allow_extrapolate(strategy),
        }
    }
}
