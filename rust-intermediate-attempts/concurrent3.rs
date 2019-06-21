use std::thread;

static NTHREADS: u32 = 10;
static NITERATIONS: u32 = 1_000_000;

struct Counter {
    val: u32,
}

fn main() {
    let mut counter = Counter { val: 0 };

    let mut children = vec![];

    for _ in 0..NTHREADS {
        children.push(thread::spawn(move || {
            for _ in 0..NITERATIONS {
                counter.val += 1;
            }
        }));
    }

    for child in children {
        let _ = child.join();
    }

    println!("{}", counter.val);
}
