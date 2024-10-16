use crossbeam_channel::{Receiver, Sender};
use operators::{types::RType, InputRef, Signal};
use waiting::{MaybeWaiting, Waiting};

pub mod hash;
pub mod operators;
pub mod waiting;

pub trait Engine {
    fn start(&self) -> impl MaybeWaiting<()>;
    fn shutdown(self) -> impl Waiting<()>;
    fn listen<T: RType>(&self, signal: Signal<T>) -> impl MaybeWaiting<Receiver<T>>;
    fn emit<T: RType>(&self, input: InputRef) -> impl MaybeWaiting<Sender<T>>;
}
