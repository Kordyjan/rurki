use rig_macros::test_suite;

#[test_suite]
pub mod sanity {
    use core::time::Duration;
    use std::thread::sleep;

    #[setup]
    fn setup(f: f32) {
        let time = Duration::from_secs_f32(f);
    }

    #[case]
    fn testing_1() {
        sleep(time * 2);
    }
}
