use rig_macros::test_suite;

#[test_suite]
pub mod input {

    use engine_base::{
        operators::input,
        waiting::{MaybeWaiting, Waiting},
        Engine,
    };

    #[setup]
    fn setup<T: Engine>(e: T) {
        let engine = e;
        let (input_ref, signal) = input::<u64>();
    }

    #[case]
    pub fn input_forwards_signal__already_running__register_on_running() {
        engine.start().wait();
        let emitter = engine.emit::<u64>(input_ref).wait();
        let listener = engine.listen(signal).wait();
        emitter.send(42)?;
        assert_eq!(listener.recv()?, 42);
    }

    #[case]
    pub fn input_forwards_signal__register_before_start() {
        let emitter = engine.emit::<u64>(input_ref).wait();
        let listener = engine.listen(signal).wait();
        emitter.send(42)?;
        engine.start().immediate();
        assert_eq!(listener.recv()?, 42);
    }

    #[case]
    pub fn input_forwards_signal__start_after_emitter_register() {
        let emitter = engine.emit::<u64>(input_ref).wait();
        engine.start().wait();
        let listener = engine.listen(signal).wait();
        emitter.send(42)?;
        assert_eq!(listener.recv()?, 42);
    }

    #[case]
    pub fn input_forwards_signal__reversed__already_running() {
        engine.start().wait();
        let listener = engine.listen(signal).wait();
        let emitter = engine.emit::<u64>(input_ref).wait();
        emitter.send(42)?;
        assert_eq!(listener.recv()?, 42);
    }

    #[case]
    pub fn input_forwards_signal__reversed__register_before_start() {
        let listener = engine.listen(signal).wait();
        let emitter = engine.emit::<u64>(input_ref).wait();
        emitter.send(42)?;
        engine.start().immediate();
        assert_eq!(listener.recv()?, 42);
    }

    #[case]
    pub fn input_forwards_signal__reversed__start_after_emitter_register() {
        let listener = engine.listen(signal).wait();
        engine.start().wait();
        let emitter = engine.emit::<u64>(input_ref).wait();
        emitter.send(42)?;
        assert_eq!(listener.recv()?, 42);
    }
}
