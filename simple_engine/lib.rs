use std::thread::{self, JoinHandle};

use commands::Command;
use crossbeam_channel::{Receiver, Sender};
use engine_base::{
    hash::Prehashed,
    operators::{types::RType, InputRef, Signal, Typed},
    Engine,
};
use internal::Impl;
use std::sync::Arc;
use transport::{Emitter, EmitterImpl, ListenerImpl};
use typed_arena::Arena;

mod commands;
mod internal;
mod transport;

pub(crate) type Apt = Arc<Prehashed<Typed>>;

pub struct SimpleEngine {
    sender: Sender<Command>,
    handle: JoinHandle<()>,
}

impl SimpleEngine {
    pub fn new() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        let handle = thread::spawn(move || {
            let internal = Impl::new();
            let arena = Arena::<Box<dyn Emitter>>::new();
            internal.run_engine(&receiver, &arena);
        });
        Self { sender, handle }
    }
}

impl Engine for SimpleEngine {
    fn start(&self) {
        self.sender.send(Command::Start).unwrap()
    }

    fn listen<T: RType>(&self, signal: Signal<T>) -> Receiver<T> {
        let (s, r) = crossbeam_channel::unbounded();
        self.sender
            .send(Command::Listen(
                signal.get_desc(),
                Box::new(ListenerImpl::new(s)),
            ))
            .unwrap();
        r
    }

    fn emit<T: RType>(&self, input: InputRef) -> Sender<T> {
        let (s, r) = crossbeam_channel::unbounded();
        self.sender
            .send(Command::Emit(
                input,
                T::into_type(),
                Box::new(EmitterImpl::new(r)),
            ))
            .unwrap();
        s
    }

    fn shutdown(self) {
        self.sender.send(Command::Shutdown).unwrap();
        self.handle.join().unwrap()
    }
}
