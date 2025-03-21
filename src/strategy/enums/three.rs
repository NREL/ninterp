use super::*;

/// See [enums module](super) documentation.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum Strategy3DEnum {
    Linear(strategy::Linear),
    Nearest(strategy::Nearest),
}

impl From<Linear> for Strategy3DEnum {
    #[inline]
    fn from(strategy: Linear) -> Self {
        Self::Linear(strategy)
    }
}

impl From<Nearest> for Strategy3DEnum {
    #[inline]
    fn from(strategy: Nearest) -> Self {
        Self::Nearest(strategy)
    }
}

impl<D> Strategy3D<D> for Strategy3DEnum
where
    D: Data + RawDataClone + Clone,
    D::Elem: Num + PartialOrd + Copy + Debug,
{
    #[inline]
    fn init(&mut self, data: &InterpData3D<D>) -> Result<(), ValidateError> {
        match self {
            Strategy3DEnum::Linear(strategy) => Strategy3D::<D>::init(strategy, data),
            Strategy3DEnum::Nearest(strategy) => Strategy3D::<D>::init(strategy, data),
        }
    }

    #[inline]
    fn interpolate(
        &self,
        data: &InterpData3D<D>,
        point: &[D::Elem; 3],
    ) -> Result<D::Elem, InterpolateError> {
        match self {
            Strategy3DEnum::Linear(strategy) => Strategy3D::<D>::interpolate(strategy, data, point),
            Strategy3DEnum::Nearest(strategy) => {
                Strategy3D::<D>::interpolate(strategy, data, point)
            }
        }
    }

    #[inline]
    fn allow_extrapolate(&self) -> bool {
        match self {
            Strategy3DEnum::Linear(strategy) => Strategy3D::<D>::allow_extrapolate(strategy),
            Strategy3DEnum::Nearest(strategy) => Strategy3D::<D>::allow_extrapolate(strategy),
        }
    }
}
