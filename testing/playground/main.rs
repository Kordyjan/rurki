use std::vec;

use rig::sanity_suite::sanity;

pub fn main() {
    let suite = sanity::suite::<vec::IntoIter<u64>, u64>();
    runner::run_tests(suite, || vec![1, 2].into_iter());
}
