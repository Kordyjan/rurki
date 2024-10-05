#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub enum Type {
    U64,
}

pub trait RType: Send + Sync + 'static {
    fn into_type() -> Type;
    fn coerce(wrapper: Wrapper) -> Self;
    fn wrap(self) -> Wrapper;
}

impl RType for u64 {
    fn into_type() -> Type {
        Type::U64
    }

    fn coerce(wrapper: Wrapper) -> Self {
        match wrapper {
            Wrapper::U64(value) => value,
        }
    }

    fn wrap(self) -> Wrapper {
        Wrapper::U64(self)
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Wrapper {
    U64(u64),
}

impl Wrapper {
    pub fn zeroed(rtype: Type) -> Self {
        match rtype {
            Type::U64 => Wrapper::U64(0),
        }
    }
}
