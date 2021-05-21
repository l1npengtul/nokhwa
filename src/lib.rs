#![deny(clippy::pedantic)]
#![warn(clippy::all)]
#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::must_use_candidate)]

pub mod backends;
mod camera;
mod camera_traits;
mod error;
mod query;
mod utils;

pub use camera_traits::*;
pub use error::NokhwaError;
pub use utils::*;
