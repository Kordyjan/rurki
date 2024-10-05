use rig_macros::test_suite;

#[test_suite]
pub mod suitest {
    use std::{thread::sleep, time::Duration};

    #[setup]
    fn setup(t: &'static f32) {
        let time = Duration::from_secs_f32(*t);
    }

    #[case]
    fn testing_1() {
        sleep(time * 2)
    }

    #[case]
    fn testing_2() {
        sleep(time * 3)
    }
}
