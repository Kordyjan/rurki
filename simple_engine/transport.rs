use crossbeam_channel::{Receiver, Select, SelectedOperation, Sender, RecvError};
use engine_base::operators::types::{RType, Wrapper};

#[derive(Debug)]
pub struct ChannelClosed;

pub trait Emitter {
    fn install<'a>(&'a self, select: &mut Select<'a>);
    fn receive(&self, op: SelectedOperation) -> Result<Wrapper, RecvError>;
}

pub trait Listener {
    fn accept(&self, wrapper: Wrapper) -> Result<(), ChannelClosed>;
}

pub struct EmitterImpl<T> {
    receiver: Receiver<T>
}

impl <T: RType> EmitterImpl<T> {
    pub fn new(receiver: Receiver<T>) -> Self {
        EmitterImpl { receiver }
    }
}

impl <T: RType> Emitter for EmitterImpl<T> {
    fn install<'a>(&'a self, select: &mut Select<'a>) {
        select.recv(&self.receiver);
    }

    fn receive(&self, op: SelectedOperation) -> Result<Wrapper, RecvError> {
        let wrapper = op.recv(&self.receiver)?.wrap();
        Ok(wrapper)
    }
}

pub struct ListenerImpl<T> {
    sender: Sender<T>,
}

impl <T: RType> ListenerImpl<T> {
    pub fn new(sender: Sender<T>) -> Self {
        ListenerImpl{
            sender,
        }
    }
}

impl <T: RType> Listener for ListenerImpl<T> {
    fn accept(&self, wrapper: Wrapper) -> Result<(), ChannelClosed> {
        self.sender.send(T::coerce(wrapper)).map_err(|_| ChannelClosed)
    }
}
