use rig_macros::test_suite;

#[test_suite]
pub mod suitest {
    use std::{thread::sleep, time::Duration};

    #[setup]
    fn setup(t: &'static f32) {}

    #[case]
    fn testing_1(t: &f32) {
        sleep(Duration::from_secs_f32(*t * 3.0))
    }

    #[case]
    fn testing_2(t: &f32) {
        sleep(Duration::from_secs_f32(*t))
    }
}
