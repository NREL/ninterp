//! Pre-defined interpolation strategies and traits for custom strategies

use super::*;

pub mod enums;
pub mod traits;

/// Linear interpolation: <https://en.wikipedia.org/wiki/Linear_interpolation>
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(Deserialize_unit_struct, Serialize_unit_struct)
)]
pub struct Linear;

/// Nearest value interpolation: <https://en.wikipedia.org/wiki/Nearest-neighbor_interpolation>
///
/// # Note
/// Float imprecision may affect the value returned near midpoints.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(Deserialize_unit_struct, Serialize_unit_struct)
)]
pub struct Nearest;

/// Left-nearest (previous value) interpolation: <https://en.wikipedia.org/wiki/Nearest-neighbor_interpolation>
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(Deserialize_unit_struct, Serialize_unit_struct)
)]
pub struct LeftNearest;

/// Right-nearest (next value) interpolation: <https://en.wikipedia.org/wiki/Nearest-neighbor_interpolation>
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(Deserialize_unit_struct, Serialize_unit_struct)
)]
pub struct RightNearest;

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    #[cfg(feature = "serde")]
    fn test_serde() {
        assert_eq!(
            serde_json::to_string(&Linear).unwrap(),
            format!("\"{}\"", stringify!(Linear))
        );
        assert_eq!(
            serde_json::to_string(&Nearest).unwrap(),
            format!("\"{}\"", stringify!(Nearest))
        );
        assert_eq!(
            serde_json::to_string(&LeftNearest).unwrap(),
            format!("\"{}\"", stringify!(LeftNearest))
        );
        assert_eq!(
            serde_json::to_string(&RightNearest).unwrap(),
            format!("\"{}\"", stringify!(RightNearest))
        );
    }
}
