#[macro_use]
extern crate bencher;
extern crate atom;

use atom::Atom;
use bencher::Bencher;
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;

fn atomic_deref_int(bench: &mut Bencher) {
    bench.iter(|| {
        let mut thread_handles = vec![];
        let atom = Arc::new(Atom::new(0));
        let n = 20;

        for _ in 0..n {
            let arc = Arc::clone(&atom);

            let thread_handle = thread::spawn(move || arc.deref());

            thread_handles.push(thread_handle);
        }

        for thread_handle in thread_handles {
            thread_handle.join().unwrap();
        }
    })
}

fn atomic_swap_int(bench: &mut Bencher) {
    bench.iter(|| {
        let mut thread_handles = vec![];
        let atom = Arc::new(Atom::new(0));
        let n = 20;

        for _ in 0..n {
            let arc = Arc::clone(&atom);

            let thread_handle = thread::spawn(move || {
                let new_value = arc.swap(&|x| x + 1);
                new_value
            });

            thread_handles.push(thread_handle);
        }

        for thread_handle in thread_handles {
            thread_handle.join().unwrap();
        }
    })
}

fn atomic_reset_int(bench: &mut Bencher) {
    bench.iter(|| {
        let mut thread_handles = vec![];
        let atom = Arc::new(Atom::new(0));
        let n = 20;

        for _ in 0..n {
            let arc = Arc::clone(&atom);

            let thread_handle = thread::spawn(move || arc.reset(99));

            thread_handles.push(thread_handle);
        }

        for thread_handle in thread_handles {
            thread_handle.join().unwrap();
        }
    })
}

fn atomic_swap_hm(bench: &mut Bencher) {
    bench.iter(|| {
        let mut thread_handles = vec![];
        let my_hm = HashMap::new();
        let atom = Arc::new(Atom::new(my_hm));
        let keys = vec![
            "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q",
            "r", "s", "t",
        ];

        for k in keys {
            let arc = Arc::clone(&atom);

            let thread_handle = thread::spawn(move || {
                let new_value = arc.swap(&|hm: &HashMap<_, _>| {
                    let mut newhm = hm.clone();
                    newhm.insert(k.to_string(), "hi".to_string());
                    hm.clone()
                });
                new_value
            });

            thread_handles.push(thread_handle);
        }

        for thread_handle in thread_handles {
            thread_handle.join().unwrap();
        }
    })
}

benchmark_group!(
    benches,
    atomic_deref_int,
    atomic_swap_int,
    atomic_reset_int,
    atomic_swap_hm
);
benchmark_main!(benches);
