#![no_std]
#![doc = include_str!("../README.md")]

#[cfg(any(feature = "syn1", feature = "syn2"))]
pub use o2o_macros::*;

pub mod traits;
