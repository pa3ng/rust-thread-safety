use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

static NTHREADS: u32 = 10;
static NITERATIONS: u32 = 1_000_000;

fn main() {
    let counter = Arc::new(AtomicUsize::new(0));

    let mut children = vec![];

    for _ in 0..NTHREADS {
        let counter = Arc::clone(&counter);
        children.push(thread::spawn(move || {
            for _ in 0..NITERATIONS {
                counter.fetch_add(1, Ordering::SeqCst);
            }
        }));
    }

    for child in children {
        let _ = child.join();
    }

    println!("{:?}", counter);
}
