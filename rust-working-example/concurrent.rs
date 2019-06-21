use std::sync::{Arc, Mutex};
use std::thread;

static NTHREADS: u32 = 10;
static NITERATIONS: u32 = 1_000_000;

struct Counter {
    val: u32,
}

impl Counter {
    fn increment_counter(&mut self) {
        self.val += 1;
    }
}

fn main() {
    let counter = Arc::new(Mutex::new(Counter { val: 0 }));

    let mut children = vec![];

    for _ in 0..NTHREADS {
        let counter = Arc::clone(&counter);
        children.push(thread::spawn(move || {
            for _ in 0..NITERATIONS {
                counter.lock().unwrap().increment_counter();
            }
        }));
    }

    for child in children {
        let _ = child.join();
    }

    println!("{}", counter.lock().unwrap().val);
}
