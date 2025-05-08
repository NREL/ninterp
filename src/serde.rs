use super::*;

pub(crate) use ndarray::{DataOwned, IntoDimension};
pub(crate) use serde::{Deserialize, Serialize};
pub(crate) use serde_unit_struct::{Deserialize_unit_struct, Serialize_unit_struct};

use serde::de::{Deserializer, Error};
use serde::ser::{SerializeSeq, Serializer};
use serde_ndim::de::MakeNDim;

#[derive(Deserialize)]
#[serde(untagged)]
#[serde(bound = "
    D::Elem: Deserialize<'de>,
")]
enum GridType<D: DataOwned> {
    VecVec(Vec<Vec<D::Elem>>),
    VecArray(Vec<ArrayBase<D, Ix1>>),
}

pub(crate) mod serde_arr_array {
    use super::*;

    pub fn serialize<D, const N: usize, Ser>(
        grid: &[ArrayBase<D, Ix1>; N],
        serializer: Ser,
    ) -> Result<Ser::Ok, Ser::Error>
    where
        D: Data + RawDataClone + Clone,
        D::Elem: Serialize + Clone,
        Ser: Serializer,
    {
        let vecs: [Vec<D::Elem>; N] = std::array::from_fn(|i| grid[i].to_vec());
        let mut seq = serializer.serialize_seq(Some(N))?;
        for vec in &vecs {
            seq.serialize_element(vec)?;
        }
        seq.end()
    }

    pub fn deserialize<'de, D, const N: usize, De>(
        deserializer: De,
    ) -> Result<[ArrayBase<D, Ix1>; N], De::Error>
    where
        D: DataOwned,
        D::Elem: Deserialize<'de> + Debug,
        De: Deserializer<'de>,
    {
        match GridType::deserialize(deserializer)? {
            GridType::VecVec(vecs) => vecs.into_iter().map(|v| v.into()).collect(),
            GridType::VecArray(arrays) => arrays,
        }
        .try_into()
        .map_err(|e| De::Error::custom(format_args!("expected {N} array(s): {e:?}")))
    }
}

pub(crate) mod serde_vec_array {
    use super::*;

    pub fn serialize<D, Ser>(
        grid: &[ArrayBase<D, Ix1>],
        serializer: Ser,
    ) -> Result<Ser::Ok, Ser::Error>
    where
        D: Data + RawDataClone + Clone,
        D::Elem: Serialize + Clone,
        Ser: Serializer,
    {
        let vecs: Vec<Vec<D::Elem>> = grid.iter().map(|arr| arr.to_vec()).collect();
        vecs.serialize(serializer)
    }

    pub fn deserialize<'de, D, De>(deserializer: De) -> Result<Vec<ArrayBase<D, Ix1>>, De::Error>
    where
        D: DataOwned,
        D::Elem: Deserialize<'de>,
        De: Deserializer<'de>,
    {
        Ok(match GridType::deserialize(deserializer)? {
            GridType::VecVec(vecs) => vecs.into_iter().map(|v| v.into()).collect(),
            GridType::VecArray(arrays) => arrays,
        })
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
#[serde(bound = "
    D::Elem: Deserialize<'de>,
    ArrayBase<D, Dim>: Deserialize<'de>,
")]
enum ValuesType<D, Dim>
where
    D: DataOwned,
    Dim: Dimension,
    ArrayBase<D, Dim>: MakeNDim<Item = D::Elem>,
{
    #[serde(deserialize_with = "serde_ndim::deserialize")]
    NDimArray(ArrayBase<D, Dim>),
    Array(ArrayBase<D, Dim>),
}

pub fn deserialize_fixed<'de, D, const N: usize, De>(
    deserializer: De,
) -> Result<ArrayBase<D, Dim<[Ix; N]>>, De::Error>
where
    D: DataOwned,
    Dim<[Ix; N]>: Dimension,
    ArrayBase<D, Dim<[Ix; N]>>: Deserialize<'de>,
    D::Elem: Deserialize<'de>,
    ArrayBase<D, Dim<[Ix; N]>>: MakeNDim<Item = D::Elem>,
    De: Deserializer<'de>,
{
    Ok(match ValuesType::deserialize(deserializer)? {
        ValuesType::NDimArray(values) => values,
        ValuesType::Array(values) => values,
    })
}

pub fn deserialize_dyn<'de, D, De>(deserializer: De) -> Result<ArrayBase<D, IxDyn>, De::Error>
where
    D: DataOwned,
    D::Elem: Deserialize<'de>,
    ArrayBase<D, IxDyn>: MakeNDim<Item = D::Elem>,
    De: Deserializer<'de>,
{
    Ok(match ValuesType::deserialize(deserializer)? {
        ValuesType::NDimArray(values) => values,
        ValuesType::Array(values) => values,
    })
}
