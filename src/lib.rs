use std::boxed::Box;
use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};

pub struct Atom<T: Clone> {
    atomic_pointer: AtomicPtr<T>,
}

unsafe impl<T: Clone> Sync for Atom<T> {}
unsafe impl<T: Clone> Send for Atom<T> {}

/*
WARNING:
This code is not safe.

All of the code that reads/writes raw pointers has
memory leaks. See: https://aturon.github.io/blog/2015/08/27/epoch/

You should not use this code. It is for exploration/demonstration.
*/

impl<T: Clone> Atom<T> {
    pub fn new(value: T) -> Atom<T> {
        let boxed = Box::new(value);
        let raw_pointer = Box::into_raw(boxed);
        Atom {
            atomic_pointer: AtomicPtr::new(raw_pointer),
        }
    }

    pub fn deref(&self) -> T {
        let current = &self.atomic_pointer.load(Ordering::SeqCst);
        unsafe { ptr::read(*current) }
    }

    pub fn reset(&self, new_value: T) -> T {
        let mut mutable_clone = new_value.clone();
        self.atomic_pointer
            .store(&mut mutable_clone, Ordering::SeqCst);
        new_value
    }

    pub fn swap(&self, f: &Fn(&T) -> T) -> T {
            loop {
                let current = &self.atomic_pointer.load(Ordering::SeqCst);
                let fn_application_result = unsafe { Box::into_raw(Box::new(f(&**current))) };
                let old = &self.atomic_pointer.compare_and_swap(
                    *current,
                    fn_application_result,
                    Ordering::SeqCst,
                );

                // if old == current, it means value was updated
                if old == current {
                    unsafe { return (*fn_application_result).clone(); }
                }
            }
    }
}

#[cfg(test)]
mod tests {
    use super::Atom;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn derefs() {
        let atom = Atom::new(5);
        assert_eq!(atom.deref(), 5);
    }

    #[test]
    fn resets() {
        let atom = Atom::new(5);
        assert_eq!(atom.reset(10), 10);
    }

    #[test]
    fn swaps() {
        let atom = Atom::new(5);
        assert_eq!(atom.swap(&|previous| previous + 1), 6);
    }

    #[test]
    fn it_swaps_with_cheap_work() {
        let mut thread_handles = vec![];
        let m = vec![];
        let results_vec: Arc<Atom<Vec<usize>>> = Arc::new(Atom::new(m));

        let n = 500;

        for _ in 0..n {
            let rv = Arc::clone(&results_vec);

            let thread_handle = thread::spawn(move || {
                rv.swap(&|v: &Vec<usize>| {
                    let mut nv = v.clone();
                    let last = nv.last().unwrap_or_else(|| &0);
                    nv.push(last + 1);
                    nv
                });
            });

            thread_handles.push(thread_handle);
        }

        for thread_handle in thread_handles {
            thread_handle.join().unwrap();
        }

        assert_eq!(results_vec.deref(), (1..501).collect::<Vec<usize>>());
    }
    #[test]
    fn it_swaps_with_more_expensive_work() {
        let mut thread_handles = vec![];

        #[derive(Clone, PartialEq, Debug)]
        struct Factorial {
            pub i: u128,
            pub results: Vec<u128>,
        }

        let results = Arc::new(Atom::new(Factorial {
            i: 32,
            results: vec![],
        }));

        let n = 32;

        for _ in 0..n {
            let rv = Arc::clone(&results);
            let thread_handle = thread::spawn(move || {
                rv.swap(&|v: &Factorial| {
                    let mut nv = v.clone();
                    let ii = v.i;
                    let new_val = v.results.last().unwrap_or_else(|| &1) * ii;
                    nv.results.push(new_val);
                    nv.i = ii - 1;
                    nv
                });
            });

            thread_handles.push(thread_handle);
        }

        for thread_handle in thread_handles {
            thread_handle.join().unwrap();
        }

        let expected = (1..33).rev().fold(vec![], |mut acc_v, x| {
            let new_val = acc_v.last().unwrap_or_else(|| &1) * x;
            acc_v.push(new_val);
            acc_v
        });
        assert_eq!(
            results.deref(),
            Factorial {
                i: 0,
                results: expected.to_vec()
            }
        );
    }
}
