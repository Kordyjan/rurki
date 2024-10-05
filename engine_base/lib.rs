use crossbeam_channel::{Receiver, Sender};
use operators::{types::RType, InputRef, Signal};

pub mod hash;
pub mod operators;

pub trait Engine {
    fn start(&self);
    fn shutdown(self);
    fn listen<T: RType>(&self, signal: Signal<T>) -> Receiver<T>;
    fn emit<T: RType>(&self, input: InputRef) -> Sender<T>;
}
