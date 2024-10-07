use engine_base::operators::{
    types::{Type, Wrapper},
    InputRef,
};

use crate::{
    transport::{Emitter, Listener},
    Apt,
};

pub enum Command {
    Start,
    Shutdown,
    Listen(Apt, Box<dyn Listener + Send>),
    Emit(InputRef, Type, Box<dyn Emitter + Send>),
}

#[derive(Debug)]
pub struct Update {
    pub input_pos: usize,
    pub value: Wrapper,
}
