use rig_macros::test_suite;

#[test_suite]
pub mod sanity {
    use engine_base::Engine;

    #[setup]
    fn setup<T: Engine>(e: T) {
        let engine = e;
    }

    #[case]
    fn engine_can_be_started_and_stopped() {
        engine.start();
        engine.shutdown();
    }
}
