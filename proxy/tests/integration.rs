//!
//! # Integration Tests.
//!

#![cfg(feature = "testmock")]

mod env;
mod knead;
mod standalone;

#[test]
fn i_ppproxy() {
    env::start_proxy();
    standalone::test();
    knead::test();
}
