use crate::hash::Prehashed;
use std::{
    marker::PhantomData,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use types::{RType, Type};

pub mod types;

type Apt = Arc<Prehashed<Typed>>;

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct Typed {
    pub desc: Desc,
    pub rtype: Type,
}

#[derive(Hash, PartialEq, Eq, Copy, Clone, Debug)]
pub struct InputRef {
    id: u64,
}

impl InputRef {
    fn new() -> Self {
        static ID_GENERATOR: AtomicU64 = AtomicU64::new(0);
        Self {
            id: ID_GENERATOR.fetch_add(1, Ordering::AcqRel),
        }
    }
}

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub enum Desc {
    Input(InputRef),
}

impl Desc {
    fn with_type<T: RType>(self) -> Typed {
        Typed {
            desc: self,
            rtype: T::into_type(),
        }
    }
}

#[derive(Clone)]
pub struct Signal<T>(Apt, PhantomData<T>);

impl<T> Signal<T> {
    pub fn get_desc(&self) -> Apt {
        Arc::clone(&self.0)
    }

    pub fn get_type(self) -> Type {
        self.0.rtype
    }
}
impl<T> From<Typed> for Signal<T> {
    fn from(desc: Typed) -> Self {
        Self(Arc::new(desc.into()), PhantomData::<T>)
    }
}

pub fn input<T: RType>() -> (InputRef, Signal<T>) {
    let input_ref = InputRef::new();
    let sig = Desc::Input(input_ref).with_type::<T>().into();
    (input_ref, sig)
}
