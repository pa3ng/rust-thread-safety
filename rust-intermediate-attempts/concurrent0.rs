use std::thread;

static NTHREADS: u32 = 10;
static NITERATIONS: u32 = 1_000_000;

fn main() {
    let counter: u32 = 0;

    let children = vec![];

    for _ in 0..NTHREADS {
        children.push(thread::spawn(|| {
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
