use std::boxed::Box;
use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};

pub struct Atom<T: Clone> {
    atomic_pointer: AtomicPtr<T>,
}

unsafe impl<T: Clone> Sync for Atom<T> {}
unsafe impl<T: Clone> Send for Atom<T> {}

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
        &self
            .atomic_pointer
            .store(&mut mutable_clone, Ordering::SeqCst);
        new_value
    }

    pub fn swap(&self, f: &Fn(&T) -> T) -> T {
        unsafe {
            loop {
                let current = &self.atomic_pointer.load(Ordering::SeqCst);
                let fn_application_result = Box::into_raw(Box::new(f(&**current)));
                let old = &self.atomic_pointer.compare_and_swap(
                    *current,
                    fn_application_result,
                    Ordering::SeqCst,
                );

                // if old == current, it means value was updated
                if old == current {
                    return (*fn_application_result).clone();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Atom;
    use rand;
    use rand::Rng;
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
    fn it_works() {
        use std::sync::Arc;

        let mut thread_handles = vec![];
        let mut i_total = 0;

        let inner = 0usize;
        let atom = Arc::new(Atom::new(inner));
        let m = vec![];
        let results_vec: Arc<Atom<Vec<usize>>> = Arc::new(Atom::new(m));

        let n = 50;

        for _ in 0..n {
            let arc = Arc::clone(&atom);
            let rv = Arc::clone(&results_vec);

            let thread_handle = thread::spawn(move || {
                let mut rng = rand::thread_rng();
                let sleep_time_ms: u64 = rng.gen_range(0, 5_000);
                std::thread::sleep(std::time::Duration::from_millis(sleep_time_ms));

                let new_value = arc.swap(&|x| x + 1);

                rv.swap(&|v: &Vec<usize>| {
                    let mut nv = v.clone();
                    let last = nv.last().unwrap_or_else(|| &0);
                    nv.push(last + 1);
                    nv
                });

                new_value
            });

            thread_handles.push(thread_handle);
        }

        for thread_handle in thread_handles {
            let val = thread_handle.join().unwrap();
            i_total = i_total + val
        }

        assert_eq!(results_vec.deref(), (1..51).collect::<Vec<usize>>());

        use std::collections::HashMap;
        let matom = Arc::new(Atom::new(HashMap::new()));

        let keys = vec!["a", "b", "c", "d"];
        let mut hm_thread_handles = vec![];

        for k in keys {
            let arc = Arc::clone(&matom);

            let thread_handle = thread::spawn(move || {
                let new_value = arc.swap(&|hm: &HashMap<_, _>| {
                    let mut newhm = hm.clone();
                    newhm.insert(k.to_string(), "hi".to_string());
                    newhm
                });

                new_value
            });

            hm_thread_handles.push(thread_handle);
        }

        for thread_handle in hm_thread_handles {
            thread_handle.join().unwrap();
        }

        let mut expected = HashMap::new();
        expected.insert("a".to_string(), "hi".to_string());
        expected.insert("b".to_string(), "hi".to_string());
        expected.insert("c".to_string(), "hi".to_string());
        expected.insert("d".to_string(), "hi".to_string());

        assert_eq!(i_total, 1275); // sum 1-50 inclusive
        assert_eq!(n, atom.deref());
        assert_eq!(expected, matom.deref());
    }
}
