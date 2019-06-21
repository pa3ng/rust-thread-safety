use std::thread;

static NTHREADS: u32 = 10;
static NITERATIONS: u32 = 1_000_000;

fn main() {
    let mut counter: u32 = 0;

    let mut children = vec![];

    for _ in 0..NTHREADS {
        children.push(thread::spawn(move || {
            for _ in 0..NITERATIONS {
                counter += 1;
            }
        }));
    }

    for child in children {
        let _ = child.join();
    }

    println!("{}", counter);
}
