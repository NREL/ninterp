use super::*;

mod n;
mod one;
mod three;
mod two;
mod zero;

pub mod data;

pub use n::{InterpND, InterpNDOwned, InterpNDViewed};
pub use one::{Interp1D, Interp1DOwned, Interp1DViewed};
pub use three::{Interp3D, Interp3DOwned, Interp3DViewed};
pub use two::{Interp2D, Interp2DOwned, Interp2DViewed};
pub use zero::Interp0D;
