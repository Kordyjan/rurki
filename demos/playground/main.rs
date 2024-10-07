use std::thread;

use engine_base::{operators::input, Engine};
use simple_engine::SimpleEngine;

pub fn main() {
    let engine = SimpleEngine::new();

    let (input_ref, input_sig) = input::<u64>();
    let listener = engine.listen(input_sig);
    let emitter = engine.emit(input_ref);

    let join_handle = thread::spawn(move || {
        let res = listener.recv().unwrap();
        println!("Received: {:?}", res);
    });

    thread::sleep(std::time::Duration::from_secs(1));
    emitter.send(42).unwrap();
    engine.start();
    join_handle.join().unwrap();
    engine.shutdown();
}
