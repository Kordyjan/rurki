use crossbeam_utils::sync::{Parker, Unparker};
use std::thread::JoinHandle;

#[must_use]
pub trait Waiting<T> {
    fn wait(self) -> T;
}

#[must_use]
pub trait MaybeWaiting<T>: Waiting<T> {
    fn immediate(self) -> T;
}

pub struct ParkWaiting<T> {
    value: T,
    parker: Parker,
}

impl<T> Waiting<T> for ParkWaiting<T> {
    fn wait(self) -> T {
        self.parker.park();
        self.value
    }
}

impl<T> ParkWaiting<T> {
    pub fn create(value: T) -> (Self, Unparker) {
        let parker = Parker::new();
        let unparker = parker.unparker().clone();
        (ParkWaiting { value, parker }, unparker)
    }
}

impl<T> MaybeWaiting<T> for ParkWaiting<T> {
    fn immediate(self) -> T {
        self.value
    }
}

pub struct ThreadJoinWaiting<T>(JoinHandle<T>);

impl<T> From<JoinHandle<T>> for ThreadJoinWaiting<T> {
    fn from(handle: JoinHandle<T>) -> Self {
        ThreadJoinWaiting(handle)
    }
}

impl<T> Waiting<T> for ThreadJoinWaiting<T> {
    fn wait(self) -> T {
        self.0.join().unwrap()
    }
}
