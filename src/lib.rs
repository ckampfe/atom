#[cfg(test)]
mod tests {
    use std::thread;
    use Atom;
    #[test]
    fn it_works() {
        use std::sync::Arc;

        let mut thread_handles = vec![];
        let mut i_total = 0;

        let atom = Arc::new(Atom::new(0));

        let n = 50;

        for _ in 0..n {
            let mut arc = Arc::clone(&atom);

            let thread_handle = thread::spawn(
                move || {
                    let new_value = arc.swap(&|x| x + 1);
                    println!("swap return: {}", new_value);
                    new_value
                }
            );

            thread_handles.push(thread_handle);
        }

        for thread_handle in thread_handles {
            let val = thread_handle.join().unwrap();
            i_total = i_total + val
        }

        println!("ATOM FINAL");
        println!("{}", atom.deref());

        use std::collections::HashMap;
        let matom = Arc::new(Atom::new(HashMap::new()));

        let keys = vec!["a", "b", "c", "d"];
        let mut hm_thread_handles = vec![];

        for k in keys {
            let mut arc = Arc::clone(&matom);

            let thread_handle = thread::spawn(
                move || {
                    let new_value = arc.swap(&|hm: &HashMap<_,_>| {

                        let mut newhm = hm.clone();
                        newhm.insert(k.to_string(), "hi".to_string());
                        println!("swap return: {:?}", newhm);

                        newhm
                    });

                    new_value
                });

            hm_thread_handles.push(thread_handle);
        }

        for thread_handle in hm_thread_handles {
            thread_handle.join().unwrap();
        }

        println!("MATOM FINAL");
        println!("{:?}", matom.deref());

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

use std::boxed::Box;
use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};

pub struct Atom<T: Clone> {
    value: AtomicPtr<T>,
}

impl<T: Clone> Atom<T> {
    pub fn new(value: T) -> Atom<T> {
        let v = Box::new(value);
        Atom {
            value: AtomicPtr::new(Box::into_raw(v)),
        }
    }

    pub fn deref(&self) -> T {
        unsafe {
            let value = &self.value;
            let t_ptr = value.load(Ordering::SeqCst);
            let v = &*t_ptr;

            v.clone()
        }
    }

    pub fn put<'a>(&self, f: &'a Fn(&T) -> T) -> () {
        unsafe {
            loop {
                let value = &self.value;
                let v = &self.value.load(Ordering::SeqCst);
                let v_val = ptr::read(v);
                let res = Box::into_raw(Box::new(f(&*v_val)));
                let p = value.compare_and_swap(*v, res, Ordering::SeqCst);

                if p == *v {
                    return ()
                }
            }
        }
    }

    pub fn swap<'a>(&self, f: &'a Fn(&T) -> T) -> T {
        unsafe {
            loop {
                let value = &self.value;
                let v = &self.value.load(Ordering::SeqCst);
                let v_val = ptr::read(v);
                let res = Box::into_raw(Box::new(f(&*v_val)));
                let p = value.compare_and_swap(*v, res, Ordering::SeqCst);

                if p == *v {
                    return (*res).clone()
                }
            }
        }
    }
}

