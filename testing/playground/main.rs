use rig::sanity_suite::sanity;

pub fn main() {
    let suite = sanity::suite();
    runner::run_tests(suite, || 0.25);
}
