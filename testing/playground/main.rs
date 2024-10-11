use std::vec;

use rig::sanity_suite::sanity;

pub fn main() {
    let suite = sanity::suite();
    runner::run_tests(suite, || vec![1u64, 2u64].into_iter());
}
