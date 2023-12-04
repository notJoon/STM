use std::ptr;
use std::{marker::PhantomData, sync::atomic::AtomicPtr};

pub struct Atomic<T> {
    /// inner atomic pointer
    inner: AtomicPtr<T>,
    _marker: PhantomData<T>,
}

impl<T> Atomic<T> {
    pub fn new(init: Option<Box<T>>) -> Self {
        Self {
            inner: AtomicPtr::new(init.map_or(ptr::null_mut(), |x| Box::into_raw(x))),
            _marker: PhantomData,
        }
    }

    /// Get a mutable reference.
    ///
    /// `None` corresponds to a null pointer.
    pub unsafe fn get_inner(&self) -> &AtomicPtr<T> {
        &self.inner
    }

    /// Get a mutable reference.
    pub unsafe fn get_inner_mut(&mut self) -> &mut AtomicPtr<T> {
        &mut self.inner
    }
}
