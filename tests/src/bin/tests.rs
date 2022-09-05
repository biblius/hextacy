//! Runs all tests with tracing
use tests::mycro_core;

pub fn main() {
    mycro_core::broker_test::broker_test("debug");
}
