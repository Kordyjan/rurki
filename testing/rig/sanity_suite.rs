use rig_macros::test_suite;

#[test_suite]
pub mod sanity {
    use core::time::Duration;
    use std::thread::sleep;

    #[setup]
    fn setup<T: Iterator<Item = S>, S>(mut f: T)
    where
        S: Into<u64>,
    {
        let time = Duration::from_secs(f.next().unwrap().into());
    }

    #[case]
    fn testing_1() {
        sleep(time * 2);
    }
}
