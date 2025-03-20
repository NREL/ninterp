use super::*;

/// See [enums module](super) documentation.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum StrategyNDEnum {
    Linear(strategy::Linear),
    Nearest(strategy::Nearest),
}

impl From<Linear> for StrategyNDEnum {
    #[inline]
    fn from(strategy: Linear) -> Self {
        StrategyNDEnum::Linear(strategy)
    }
}

impl From<Nearest> for StrategyNDEnum {
    #[inline]
    fn from(strategy: Nearest) -> Self {
        StrategyNDEnum::Nearest(strategy)
    }
}

impl<D> StrategyND<D> for StrategyNDEnum
where
    D: Data + RawDataClone + Clone,
    D::Elem: Num + PartialOrd + Copy + Debug,
{
    #[inline]
    fn init(&mut self, data: &InterpDataND<D>) -> Result<(), ValidateError> {
        match self {
            StrategyNDEnum::Linear(strategy) => StrategyND::<D>::init(strategy, data),
            StrategyNDEnum::Nearest(strategy) => StrategyND::<D>::init(strategy, data),
        }
    }

    #[inline]
    fn interpolate(
        &self,
        data: &InterpDataND<D>,
        point: &[D::Elem],
    ) -> Result<D::Elem, InterpolateError> {
        match self {
            StrategyNDEnum::Linear(strategy) => StrategyND::<D>::interpolate(strategy, data, point),
            StrategyNDEnum::Nearest(strategy) => {
                StrategyND::<D>::interpolate(strategy, data, point)
            }
        }
    }

    #[inline]
    fn allow_extrapolate(&self) -> bool {
        match self {
            StrategyNDEnum::Linear(strategy) => StrategyND::<D>::allow_extrapolate(strategy),
            StrategyNDEnum::Nearest(strategy) => StrategyND::<D>::allow_extrapolate(strategy),
        }
    }
}
