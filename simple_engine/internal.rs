use std::{collections::VecDeque, sync::Arc};

use crossbeam_channel::{Receiver, Select, SelectedOperation};
use engine_base::operators::{types::Wrapper, Desc::Input, InputRef, Typed};
use rustc_hash::FxHashMap;
use typed_arena::Arena;

use crate::{
    commands::{Command, Update},
    transport::{Emitter, Listener},
    Apt,
};

pub struct Impl<'a> {
    fields: Vec<Wrapper>,
    listeners: Vec<Option<Box<dyn Listener>>>,
    signals: FxHashMap<Apt, usize>,
    inputs: FxHashMap<InputRef, usize>,
    emitters: Vec<&'a dyn Emitter>,
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

        let mut running = true;
        loop {
            let op = select.select();
            if op.index() == 0 {
                let command = op.recv(receiver);
                match command {
                    Ok(Command::Start) => {
                        break;
                    }
                    Ok(Command::Shutdown) => {
                        running = false;
                        break;
                    }
                    Ok(Command::Listen(signal, listener)) => {
                        self.add_listener(signal, listener);
                    }
                    Ok(Command::Emit(iref, rtype, emitter)) => {
                        let ptr = arena.alloc(emitter);
                        self.emitters.push(&**ptr);
                        ptr.install(&mut select);
                        let field = self.get_signal_id(Arc::new(
                            Typed {
                                desc: Input(iref),
                                rtype,
                            }
                            .into(),
                        ));
                        self.inputs.insert(iref, field);
                        self.emitters_to_fields.push(field);
                    }
                    Err(_) => break,
                }
            } else {
                let update = self.create_update(op);
                self.prestart_queue.push_back(update);
            }
        }
        if running {
            self.drain_queue();
            self.work(&mut select, receiver, &arena);
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
                    Ok(Command::Start) => {
                        panic!("Engine already started!")
                    }
                    Ok(Command::Shutdown) => {
                        break;
                    }
                    Ok(Command::Listen(signal, listener)) => {
                        self.add_listener(signal, listener);
                    }
                    Ok(Command::Emit(iref, rtype, emitter)) => {
                        let ptr = arena.alloc(emitter);
                        self.emitters.push(&**ptr);
                        ptr.install(select);
                        let field = self.get_signal_id(Arc::new(
                            Typed {
                                desc: Input(iref),
                                rtype,
                            }
                            .into(),
                        ));
                        self.inputs.insert(iref, field);
                        self.emitters_to_fields.push(field);
                    }
                    Err(_) => break,
                }
            } else {
                let update = self.create_update(op);
                self.update(update);
            }
        }
    }

    fn drain_queue(&mut self) {
        while let Some(update) = self.prestart_queue.pop_front() {
            self.update(update);
        }
    }

    fn create_update(&mut self, op: SelectedOperation) -> Update {
        let id = op.index() - 1;
        let value = self.emitters[id].receive(op).unwrap();
        let input_pos = self.emitters_to_fields[id];
        Update { input_pos, value }
    }

    fn update(&mut self, Update { input_pos, value }: Update) {
        self.fields[input_pos] = value.clone();
        if let Some(callback) = self.listeners[input_pos].as_ref() {
            callback.accept(value).unwrap();
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
            };
            self.signals.insert(signal.clone(), res);
            res
        }
    }
}
