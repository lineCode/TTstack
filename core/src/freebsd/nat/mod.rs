#[cfg(not(feature = "testmock"))]
pub(crate) mod real;
#[cfg(not(feature = "testmock"))]
pub(crate) use real::*;

#[cfg(feature = "testmock")]
pub(crate) mod mocker;
#[cfg(feature = "testmock")]
pub(crate) use mocker::*;
