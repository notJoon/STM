use std::{
    mem,
    sync::atomic::{AtomicPtr, Ordering},
    thread,
};

static BLOCKED: u8 = 0x01;
static FREE: u8 = 0x02;
static DEAD: u8 = 0x03;

#[derive(Debug, PartialEq)]
pub enum State {
    /// hazard pointer does not protect any object
    Free,
    /// when object is dead, it does not need to be protected
    /// also, can be deallocated if needed
    ///
    /// Once a hazard enters this state, it's important not to use it further.
    /// For instance, altering its state or similar actions should be avoided,
    /// as its behavior is not clearly defined and it might have already been deallocated.
    Dead,
    /// hazard pointer protects a object
    ///
    /// `Protect` ensures that the pointer it refers to isn't deleted while the hazard remains in this state
    Protect(*const u8),
}

/// Instantiate a new hazard reader-writer pair.
///
/// This action generates a new hazard pair in a blocked state.
///
/// Each end of the hazard shares a reference to its state, equivalent to State, but encoded for atomic access.
///
/// Additionally, there's a 'Blocked' state. When the hazard is in this state,
/// any read operation will be on hold until it's unblocked.
pub fn create() -> (Reader, Writer) {
    let ptr = unsafe {
        Box::into_raw(Box::new(AtomicPtr::new(&BLOCKED as *const u8 as *mut u8)))
            .as_ref()
            .unwrap()
    };

    let reader = Reader { ptr };
    let writer = Writer { ptr };

    (reader, writer)
}

#[derive(Debug)]
pub struct Reader {
    ptr: &'static AtomicPtr<u8>,
}

impl Reader {
    pub fn get(&self) -> State {
        // counts the number of spins
        let mut _spins = 0;

        // spin until not blocked
        loop {
            let ptr = self.ptr.load(Ordering::Acquire) as *const u8;

            if ptr == &BLOCKED as *const u8 {
                _spins += 1;
                continue;
            } else if ptr == &FREE as *const u8 {
                return State::Free;
            } else if ptr == &DEAD as *const u8 {
                return State::Dead;
            } else {
                return State::Protect(ptr);
            }
        }
    }

    /// destroy the hazard pointer
    /// 
    /// # Safety
    /// 
    /// This operation is considered unsafe because it assumes that 
    /// the writer component is no longer active or in use. 
    /// 
    /// Since the type system cannot currently enforce this condition, 
    /// it's crucial that the caller ensures this is the case
    pub unsafe fn destroy(self) {
        if self.get() != State::Dead {
            panic!("hazard pointer is not dead");
        }

        // load a pointer and deallocate it
        drop(Box::from_raw(
            self.ptr as *const AtomicPtr<u8> as *mut AtomicPtr<u8>,
        ));
        // ensure the pointer is not used again
        mem::forget(self);
    }
}

impl Drop for Reader {
    fn drop(&mut self) {
        panic!("hazard pointer is not dead");
    }
}

#[derive(Debug)]
pub struct Writer {
    ptr: &'static AtomicPtr<u8>,
}

impl Writer {
    pub fn is_blocked(&self) -> bool {
        self.ptr.load(Ordering::Acquire) == &BLOCKED as *const u8 as *mut u8
    }

    /// block the hazard pointer
    pub fn block(&self) {
        self.ptr
            .store(&BLOCKED as *const u8 as *mut u8, Ordering::Release);
    }

    /// set the hazard pointer state to free
    pub fn free(&self) {
        self.ptr
            .store(&FREE as *const u8 as *mut u8, Ordering::Release);
    }

    /// protect a pointer
    pub fn protect(&self, ptr: *const u8) {
        self.ptr.store(ptr as *mut u8, Ordering::Release);
    }

    /// set the hazard pointer state to dead
    /// 
    /// # Safety
    /// 
    /// This approach is unsafe because using the system after this call breaks invariants. 
    /// To maintain safety within the type system, use `Writer::kill()`.
    unsafe fn dead(&self) {
        self.ptr
            .store(&DEAD as *const u8 as *mut u8, Ordering::Release);
    }

    /// set the hazard pointer state to dead
    pub fn kill(self) {
        unsafe {
            self.dead();
        }
        mem::forget(self);
    }
}

impl Drop for Writer {
    fn drop(&mut self) {
        if !thread::panicking() {
            panic!("hazard pointer is not dead");
        }

        unsafe {
            self.dead();
        }
    }
}
