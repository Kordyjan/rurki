use rig_macros::test_suite;

#[test_suite]
pub mod input {

    use engine_base::{operators::input, Engine};

    #[setup]
    fn setup<T: Engine>(e: T) {
        let engine = e;
        let (input_ref, signal) = input::<u64>();
    }

    #[case]
    pub fn input_forwards_signal__already_running__register_on_running() {
        engine.start();
        let emitter = engine.emit::<u64>(input_ref);
        let listener = engine.listen(signal);
        emitter.send(42);
        assert_eq!(listener.recv().unwrap(), 42);
    }

    #[case]
    pub fn input_forwards_signal__register_before_start() {
        let emitter = engine.emit::<u64>(input_ref);
        let listener = engine.listen(signal);
        emitter.send(42);
        engine.start();
        assert_eq!(listener.recv().unwrap(), 43);
    }

    #[case]
    pub fn input_forwards_signal__start_after_emitter_register() {
        let emitter = engine.emit::<u64>(input_ref);
        engine.start();
        let listener = engine.listen(signal);
        emitter.send(42);
        assert_eq!(listener.recv().unwrap(), 42);
    }

    #[case]
    pub fn input_forwards_signal__reversed__already_running() {
        engine.start();
        let listener = engine.listen(signal);
        let emitter = engine.emit::<u64>(input_ref);
        emitter.send(42);
        assert_eq!(listener.recv().unwrap(), 42);
    }

    #[case]
    pub fn input_forwards_signal__reversed__register_before_start() {
        let listener = engine.listen(signal);
        let emitter = engine.emit::<u64>(input_ref);
        emitter.send(42);
        engine.start();
        assert_eq!(listener.recv().unwrap(), 42);
    }

    #[case]
    pub fn input_forwards_signal__reversed__start_after_emitter_register() {
        let listener = engine.listen(signal);
        engine.start();
        let emitter = engine.emit::<u64>(input_ref);
        emitter.send(42);
        assert_eq!(listener.recv().unwrap(), 42);
    }
}
