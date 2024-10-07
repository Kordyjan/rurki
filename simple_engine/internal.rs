use std::{collections::VecDeque, sync::Arc};

use crossbeam_channel::{Receiver, RecvError, Select, SelectedOperation};
use crossbeam_utils::sync::Unparker;
use engine_base::operators::{
    types::Wrapper,
    Desc::{Add, Input},
    InputRef, Typed,
};
use rustc_hash::FxHashMap;
use typed_arena::Arena;

use crate::{
    commands::{Command, Update},
    transport::{Emitter, Listener},
    Apt,
};

type RecvResult<T> = Result<T, RecvError>;

pub struct Impl<'a> {
    fields: Vec<Wrapper>,
    listeners: Vec<Option<Box<dyn Listener>>>,
    signals: FxHashMap<Apt, usize>,
    inputs: FxHashMap<InputRef, usize>,
    emitters: Vec<Option<&'a dyn Emitter>>,
    emitters_to_fields: Vec<usize>,
    prestart_queue: VecDeque<Update>,
}

impl<'a> Impl<'a> {
    pub fn new() -> Self {
        Self {
            fields: Vec::default(),
            listeners: Vec::default(),
            signals: FxHashMap::default(),
            inputs: FxHashMap::default(),
            emitters: Vec::default(),
            emitters_to_fields: Vec::default(),
            prestart_queue: VecDeque::new(),
        }
    }

    pub fn run_engine(mut self, receiver: &Receiver<Command>, arena: &'a Arena<Box<dyn Emitter>>) {
        let mut select = Select::new();
        select.recv(receiver);

        let unparker_or_shutdown: Option<Unparker> = loop {
            let op = select.select();
            if op.index() == 0 {
                let command = op.recv(receiver);
                match command {
                    Ok(Command::Start(unparker)) => {
                        break Some(unparker);
                    }
                    Ok(Command::Shutdown) => {
                        break None;
                    }
                    Ok(Command::Listen {
                        signal,
                        listener,
                        unparker,
                    }) => {
                        self.add_listener(signal, listener);
                        unparker.unpark();
                    }
                    Ok(Command::Emit {
                        input,
                        rtype,
                        emitter,
                        unparker,
                    }) => {
                        let ptr = arena.alloc(emitter);
                        self.emitters.push(Some(&**ptr));
                        ptr.install(&mut select);
                        let field = self.get_signal_id(Arc::new(
                            Typed {
                                desc: Input(input),
                                rtype,
                            }
                            .into(),
                        ));
                        self.inputs.insert(input, field);
                        self.emitters_to_fields.push(field);
                        unparker.unpark();
                    }
                    Err(_) => break None,
                }
            } else {
                let index = op.index();
                if let Ok(update) = self.create_update(op) {
                    self.prestart_queue.push_back(update);
                } else {
                    select.remove(index);
                    self.emitters[index - 1] = None;
                }
            }
        };
        if let Some(unparker) = unparker_or_shutdown {
            self.drain_queue();
            unparker.unpark();
            self.work(&mut select, receiver, arena);
        }
    }
    fn work(
        mut self,
        select: &'a mut Select<'a>,
        receiver: &'a Receiver<Command>,
        arena: &'a Arena<Box<dyn Emitter>>,
    ) {
        loop {
            let op = select.select();
            if op.index() == 0 {
                match op.recv(receiver) {
                    Ok(Command::Start(unparker)) => {
                        unparker.unpark();
                        eprintln!("Engine start called on running engine!");
                    }
                    Ok(Command::Shutdown) => {
                        break;
                    }
                    Ok(Command::Listen {
                        signal,
                        listener,
                        unparker,
                    }) => {
                        self.add_listener(signal, listener);
                        unparker.unpark();
                    }
                    Ok(Command::Emit {
                        input,
                        rtype,
                        emitter,
                        unparker,
                    }) => {
                        let ptr = arena.alloc(emitter);
                        self.emitters.push(Some(&**ptr));
                        ptr.install(select);
                        let field = self.get_signal_id(Arc::new(
                            Typed {
                                desc: Input(input),
                                rtype,
                            }
                            .into(),
                        ));
                        self.inputs.insert(input, field);
                        self.emitters_to_fields.push(field);
                        unparker.unpark();
                    }
                    Err(_) => break,
                }
            } else {
                let index = op.index();
                if let Ok(update) = self.create_update(op) {
                    self.update(update);
                } else {
                    select.remove(index);
                    self.emitters[index - 1] = None;
                }
            }
        }
    }

    fn drain_queue(&mut self) {
        while let Some(update) = self.prestart_queue.pop_front() {
            self.update(update);
        }
    }

    fn create_update(&mut self, op: SelectedOperation) -> RecvResult<Update> {
        let id = op.index() - 1;
        let value = self.emitters[id]
            .expect("Emitter already discarded")
            .receive(op)?;
        let input_pos = self.emitters_to_fields[id];
        Ok(Update { input_pos, value })
    }

    fn update(&mut self, Update { input_pos, value }: Update) {
        self.fields[input_pos] = value.clone();
        let needs_removal = if let Some(callback) = self.listeners[input_pos].as_ref() {
            !matches!(callback.accept(value), Ok(()))
        } else {
            false
        };
        if needs_removal {
            self.listeners[input_pos] = None;
        }
    }

    fn add_listener(&mut self, signal: Apt, listener: Box<dyn Listener>) {
        let id = self.get_signal_id(signal);
        self.listeners[id] = Some(listener);
    }

    fn get_signal_id(&mut self, signal: Apt) -> usize {
        if let Some(id) = self.signals.get(&signal) {
            *id
        } else {
            let Typed { desc, rtype } = &**signal;
            let res = match desc {
                Input(input) => {
                    self.fields.push(Wrapper::zeroed(*rtype));
                    self.listeners.push(None);
                    let res = self.fields.len() - 1;
                    self.inputs.insert(*input, res);
                    res
                }
                Add(left, right) => {
                    let left_id = self.get_signal_id(left.clone());
                    let right_id = self.get_signal_id(right.clone());
                    let left = &self.fields[left_id];
                    let right = &self.fields[right_id];

                    let new = left.add(right);
                    self.fields.push(new);
                    self.listeners.push(None);
                    let res = self.fields.len() - 1;

                    res
                }
            };
            self.signals.insert(signal.clone(), res);
            res
        }
    }
}
