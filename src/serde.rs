use super::*;

pub(crate) use ndarray::{DataOwned, IntoDimension};
pub(crate) use serde::{Deserialize, Serialize};
pub(crate) use serde_unit_struct::{Deserialize_unit_struct, Serialize_unit_struct};

#[cfg(feature = "serde-simple")]
pub(crate) mod serde_arr_array {
    use super::*;
    use serde::de::{Deserializer, Error};
    use serde::ser::{SerializeSeq, Serializer};

    pub fn serialize<S, D, const N: usize>(
        grid: &[ArrayBase<D, Ix1>; N],
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        D: Data + RawDataClone + Clone,
        D::Elem: Serialize + Clone,
    {
        let vecs: [Vec<D::Elem>; N] = std::array::from_fn(|i| grid[i].to_vec());
        let mut seq = serializer.serialize_seq(Some(N))?;
        for vec in &vecs {
            seq.serialize_element(vec)?;
        }
        seq.end()
    }

    pub fn deserialize<'de, D, De, const N: usize>(
        deserializer: De,
    ) -> Result<[ArrayBase<D, Ix1>; N], De::Error>
    where
        De: Deserializer<'de>,
        D: DataOwned,
        D::Elem: Deserialize<'de>,
    {
        let items: Vec<Vec<D::Elem>> = Deserialize::deserialize(deserializer)?;
        let arrays: Vec<ArrayBase<D, Ix1>> = items.into_iter().map(|v| v.into()).collect();
        arrays
            .try_into()
            .map_err(|_| De::Error::custom(format_args!("expected {} arrays", N)))
    }
}

#[cfg(feature = "serde-simple")]
pub(crate) mod serde_vec_array {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S, D>(grid: &[ArrayBase<D, Ix1>], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        D: Data + RawDataClone + Clone,
        D::Elem: Serialize + Clone,
    {
        let vecs: Vec<Vec<D::Elem>> = grid.iter().map(|arr| arr.to_vec()).collect();
        vecs.serialize(serializer)
    }

    pub fn deserialize<'de, D, De>(deserializer: De) -> Result<Vec<ArrayBase<D, Ix1>>, De::Error>
    where
        De: Deserializer<'de>,
        D: DataOwned,
        D::Elem: Deserialize<'de>,
    {
        let items = Vec::<Vec<D::Elem>>::deserialize(deserializer)?;
        let arrays = items.into_iter().map(|v| v.into()).collect();
        Ok(arrays)
    }
}
