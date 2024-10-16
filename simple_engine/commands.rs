use std::fmt::{self, Debug, Formatter};

use crossbeam_utils::sync::Unparker;
use engine_base::operators::{
    types::{Type, Wrapper},
    InputRef,
};

use crate::{
    transport::{Emitter, Listener},
    Apt,
};

pub enum Command {
    Start(Unparker),
    Shutdown,
    Listen {
        signal: Apt,
        listener: Box<dyn Listener + Send>,
        unparker: Unparker,
    },
    Emit {
        input: InputRef,
        rtype: Type,
        emitter: Box<dyn Emitter + Send>,
        unparker: Unparker,
    },
}

impl Debug for Command {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Command::Start(_) => write!(f, "Start")?,
            Command::Shutdown => write!(f, "Shutdown")?,
            Command::Listen {
                signal, listener, ..
            } => write!(f, "Listen({signal:?}, {listener:p})")?,
            Command::Emit {
                input,
                rtype,
                emitter,
                ..
            } => write!(f, "Emit({input:?}, {rtype:?}, {emitter:p})")?,
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Update {
    pub input_pos: usize,
    pub value: Wrapper,
}
