#[cfg(test)]
mod hazard_tests {
    use std::{ptr, thread};

    use STM::hazard::{create, State};

    #[test]
    fn test_set_and_get() {
        let (r, w) = create();
        assert!(w.is_blocked());

        w.free();
        assert!(!w.is_blocked());
        assert_eq!(r.get(), State::Free);

        w.free();
        assert!(!w.is_blocked());
        assert_eq!(r.get(), State::Free);

        let x = 42;

        w.protect(&x);
        assert_eq!(r.get(), State::Protect(&x));

        w.protect(ptr::null());
        assert_eq!(r.get(), State::Protect(ptr::null()));

        w.protect(0x1 as *const u8);
        assert_eq!(r.get(), State::Protect(0x1 as *const u8));

        w.kill();
        unsafe {
            r.destroy();
        }
    }

    #[test]
    fn hazard_pair() {
        let (r, w) = create();
        let x = 2;

        w.free();
        assert_eq!(r.get(), State::Free);
        w.protect(&x);
        assert_eq!(r.get(), State::Protect(&x));
        w.kill();
        assert_eq!(r.get(), State::Dead);

        unsafe {
            r.destroy();
        }
    }

    #[test]
    fn cross_thread() {
        for _ in 0..64 {
            let (r, w) = create();

            thread::spawn(move || {
                w.kill();
            }).join().unwrap();

            assert_eq!(r.get(), State::Dead);
            unsafe { r.destroy(); }
        }
    }

    #[test]
    fn drop() {
        for _ in 0..9000 {
            let (r, w) = create();
            w.kill();
            unsafe {
                r.destroy();
            }
        }
    }
}
