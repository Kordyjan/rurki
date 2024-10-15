use std::fmt::{self, Debug, Formatter};

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


impl Debug for Command {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Command::Start => write!(f, "Start")?,
            Command::Shutdown => write!(f, "Shutdown")?,
            Command::Listen(signal, listener) => {
                write!(f, "Listen({:?}, {:p})", signal, listener)?
            }
            Command::Emit(input, ty, emitter) => {
                write!(f, "Emit({:?}, {:?}, {:p})", input, ty, emitter)?
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Update {
    pub input_pos: usize,
    pub value: Wrapper,
}
