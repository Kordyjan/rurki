use rig_macros::test_suite;

#[test_suite]
pub mod sanity {
    use engine_base::{waiting::{MaybeWaiting, Waiting}, Engine};

    #[setup]
    fn setup<T: Engine>(e: T) {
        let engine = e;
    }
    #[case]
    fn engine_can_be_started_and_stopped() {
        engine.start().wait();
        engine.shutdown().wait();
    }

    #[case]
    fn engine_can_be_started_and_stopped_without_waiting() {
        engine.start().immediate();
        engine.shutdown().wait();
    }
}
