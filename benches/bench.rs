#[macro_use]
extern crate bencher;
extern crate atom;

use atom::Atom;
use bencher::Bencher;
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;

// fn empty(bench: &mut Bencher) {
//     bench.iter(|| 1)
// }
//
// fn decode(bench: &mut Bencher) {
//     let document = "d5:howdyi2848242e2:ok4:finee";
//     bench.iter(|| {
//         //for _ in 1..10000 {
//         document.decode();
//         // }
//         // (0..1000).fold(0, |x, y| x + y)
//     })
// }
//
// fn encode(bench: &mut Bencher) {
//     let mut hm = HashMap::new();
//     hm.insert("howdy".to_string(), Int(2848242));
//     hm.insert("ok".to_string(), Str("fine".to_string()));
//     bench.iter(|| {
//         hm.encode();
//     });
// }

fn atomic_swap_int(bench: &mut Bencher) {
    bench.iter(|| {

        let mut thread_handles = vec![];
        let atom = Arc::new(Atom::new(0));
        let n = 20;

        for _ in 0..n {
            let mut arc = Arc::clone(&atom);

            let thread_handle = thread::spawn(move || {
                let new_value = arc.swap(&|x| x + 1);
                // println!("swap return: {}", new_value);
                new_value
            });

            thread_handles.push(thread_handle);
        }

        for thread_handle in thread_handles {
            thread_handle.join().unwrap();
        }
    })
}

fn atomic_put_int(bench: &mut Bencher) {
    bench.iter(|| {

        let mut thread_handles = vec![];
        let atom = Arc::new(Atom::new(0));
        let n = 20;

        for _ in 0..n {
            let mut arc = Arc::clone(&atom);

            let thread_handle = thread::spawn(move || {
                let new_value = arc.put(&|x| x + 1);
                // println!("put return: {}", new_value);
                new_value
            });

            thread_handles.push(thread_handle);
        }

        for thread_handle in thread_handles {
            thread_handle.join().unwrap();
        }
    })
}

fn atomic_put_hm(bench: &mut Bencher) {
    bench.iter(|| {

        let mut thread_handles = vec![];
        let my_hm = HashMap::new();
        let atom = Arc::new(Atom::new(my_hm));
        let keys = vec![
            "a", "b", "c", "d",
            "e", "f", "g", "h",
            "i", "j", "k", "l",
            "m", "n", "o", "p",
            "q", "r", "s", "t"
         ];

        for k in keys {
            let mut arc = Arc::clone(&atom);

            let thread_handle = thread::spawn(move || {
                let new_value = arc.swap(&|hm: &HashMap<_,_>| {
                    let mut newhm = hm.clone();
                    newhm.insert(k.to_string(), "hi".to_string());
                    hm.clone()
                });
                // println!("put return: {}", new_value);
                new_value
            });

            thread_handles.push(thread_handle);
        }

        for thread_handle in thread_handles {
            thread_handle.join().unwrap();
        }
    })
}

benchmark_group!(benches, atomic_swap_int, atomic_put_int, atomic_put_hm);
benchmark_main!(benches);
