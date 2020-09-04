//!
//! # Integration Tests.
//!

#![cfg(feature = "testmock")]

mod env;
mod knead;
mod standalone;

#[test]
fn i_ppserver() {
    env::start_server();
    standalone::test();
    knead::test();
}
