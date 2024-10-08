use rig_macros::test_suite;
use runner::model::Test;

#[test_suite]
pub mod suitest {
    use std::{thread::sleep, time::Duration};

    #[setup]
    fn setup(t: f32) {
        let time = Duration::from_secs_f32(t);
    }

    #[case]
    fn testing_1() {
        sleep(time * 2);
    }

    #[case]
    fn testing_2() {
        sleep(time * 3);
    }
}

#[test_suite]
pub mod suitest2 {
    use std::{thread::sleep, time::Duration};

    #[setup]
    fn setup(t: f32) {
        let time = Duration::from_secs_f32(t);
    }

    #[case]
    fn testing_1() {
        sleep(time * 2);
    }

    #[case]
    fn testing_2() {
        sleep(time * 3);
    }
}

pub fn composite() -> Test<f32> {
    Test::Suite {
        name: "Composite".to_string(),
        tests: vec![suitest::suite(), suitest2::suite()],
    }
}
