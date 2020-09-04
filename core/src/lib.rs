//!
//! # PP 核心实现
//!
//! VM will run faster on FreeBSD,
//! especially in IO-about operations.
//!

mod def;
pub use def::*;

#[cfg(target_os = "freebsd")]
mod freebsd;
#[cfg(target_os = "freebsd")]
pub use freebsd::*;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::*;

mod test;

#[cfg(not(feature = "testmock"))]
#[cfg(feature = "zfs_snapshot")]
const CLONE_MARK: &str = "clone_";
