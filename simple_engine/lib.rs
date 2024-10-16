use std::thread::{self, JoinHandle};

use commands::Command;
use crossbeam_channel::{Receiver, Sender};
use engine_base::{
    hash::Prehashed,
    operators::{types::RType, InputRef, Signal, Typed},
    waiting::{MaybeWaiting, ParkWaiting, ThreadJoinWaiting, Waiting},
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
    fn start(&self) -> impl MaybeWaiting<()> {
        let (wait, unparker) = ParkWaiting::create(());
        self.sender
            .send(Command::Start(unparker))
            .expect("Engine thread is dead");
        wait
    }

    fn listen<T: RType>(&self, signal: Signal<T>) -> impl MaybeWaiting<Receiver<T>> {
        let (s, r) = crossbeam_channel::unbounded();
        let (wait, unparker) = ParkWaiting::create(r);
        self.sender
            .send(Command::Listen {
                signal: signal.get_desc(),
                listener: Box::new(ListenerImpl::new(s)),
                unparker,
            })
            .expect("Engine thread is dead");
        wait
    }

    fn emit<T: RType>(&self, input: InputRef) -> impl MaybeWaiting<Sender<T>> {
        let (s, r) = crossbeam_channel::unbounded();
        let (wait, unparker) = ParkWaiting::create(s);
        self.sender
            .send(Command::Emit {
                input,
                rtype: T::into_type(),
                emitter: Box::new(EmitterImpl::new(r)),
                unparker,
            })
            .expect("Engine thread is dead");
        wait
    }

    fn shutdown(self) -> impl Waiting<()> {
        self.sender
            .send(Command::Shutdown)
            .expect("Engine thread is dead");

        ThreadJoinWaiting::from(self.handle)
    }
}

impl Default for SimpleEngine {
    fn default() -> Self {
        Self::new()
    }
}
