use rig_macros::test_suite;

#[test_suite]
pub mod suitest {
    use std::{thread::sleep, time::Duration};

    fn something_other() {
        println!("not a test");
    }

    #[case]
    fn testing_1() {
        sleep(Duration::from_secs_f32(0.75))
    }

    #[case]
    fn testing_2() {
        sleep(Duration::from_secs_f32(0.25))
    }
}
