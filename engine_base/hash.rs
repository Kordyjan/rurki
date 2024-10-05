use rustc_hash::FxBuildHasher;
use std::{
    hash::{BuildHasher, Hash, Hasher},
    ops::{Deref, DerefMut},
};

#[derive(Debug, Clone)]
pub struct Prehashed<T> {
    inner: T,
    hash: u64,
}

impl<T> From<T> for Prehashed<T>
where
    T: Hash,
{
    fn from(inner: T) -> Self {
        let hash = FxBuildHasher.hash_one(&inner);
        Self { inner, hash }
    }
}

impl<T> Deref for Prehashed<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for Prehashed<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T> AsRef<T> for Prehashed<T> {
    fn as_ref(&self) -> &T {
        &self.inner
    }
}

impl<T> Hash for Prehashed<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash);
    }
}

impl<T: PartialEq> PartialEq for Prehashed<T> {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash && self.inner == other.inner
    }
}

impl<T: Eq> Eq for Prehashed<T> {}
