use super::*;

pub mod data;

pub mod n;
pub mod one;
pub mod three;
pub mod two;
pub mod zero;

pub use n::{InterpND, InterpNDOwned, InterpNDViewed};
pub use one::{Interp1D, Interp1DOwned, Interp1DViewed};
pub use three::{Interp3D, Interp3DOwned, Interp3DViewed};
pub use two::{Interp2D, Interp2DOwned, Interp2DViewed};
pub use zero::Interp0D;
