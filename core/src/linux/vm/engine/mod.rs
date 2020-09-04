//!
//! # VM Engine
//!

#[cfg(not(feature = "testmock"))]
mod real;
#[cfg(not(feature = "testmock"))]
pub(in crate::linux) use real::*;

#[cfg(feature = "testmock")]
mod mocker;
#[cfg(feature = "testmock")]
pub(in crate::linux) use mocker::*;
