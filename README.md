# Concurrency...

Having just taken a look at a simple Rust RESTful web service, let's explore another Rust's (and Java's) behavior with respect to concurrency. The goal is simple: in each of  10 concurrent threads, increment a _shared_ counter 1,000,000 times. When all threads complete, the counter (which is initialized to 0) should equal 10,000,000.

## [`java-first-attempt`](java-first-attempt/Concurrent.java)

Our first attempt with Java is pretty straightforward. It uses a `volatile boolean started` to keep a thread from completing before another starts, in increments a `volatile int counter` using `counter++` in each thread. It does not behave as expected:

```
$ java Concurrent 
2320572
```

What is going on? The volatile -- important for `started` -- is a ruse for our `counter`. Java doesn't prevent race conditions, nor does it care what happens when two threads attempt to access and, possibly, mutate a variable. Note that Java's not catching race conditions during compilation is understandable, but the lack of runtime exceptions results in unpredictable behavior that's often hard to diagnose.

## [`java-working-example`](java-working-example/Concurrent.java)

The solution is fairly simple: guard the mutation via `synchronized` access, which guarantees a single thread's access at a time. This involves a new method that is called by the threads:

```java
  synchronized void incrementCounter() {
    counter += 1;
  }
```

The result with this change is what we expect:

```
$ java Concurrent 
10000000
```

## [`rust-first-attempt`](rust-first-attempt/concurrent.rs)

We'll build our first Rust variant by stealing from [rust-by-example](https://doc.rust-lang.org/rust-by-example/std_misc/threads.html). The example is pretty clear (the code is unsurprising at first glance) with the exceptions being the `mut` and `move` keywords, which we borrowed from the example. We'll get back to that, and instead see what happens when we run it:

```
$  ./concurrent 
0
```

`0`?! At least it's not random :/ ...

## [`rust-intermediate-attempts`](rust-intermediate-attempts)

Let's explore what's going on an in the process learn a lot about the language. We'll do so by trying a bunch of things, seeing what happens, and making adjustments as we go along. By the time we get to a working example, you'll understand Rust's memory model and its mutability handling.

### [`concurrent0.rs`](rust-intermediate-attempts/concurrent0.rs)

Let's start by removing the `mut` and `move` keywords to see what, if anything, they do. Compiling (note: the output is snipped for brevity, try compiling yourself to see complete messages):

```
$ rustc concurrent0.rs
<snip>
9  |     let children = vec![];
   |         -------- help: make this binding mutable: `mut children`
<snip>
help: to force the closure to take ownership of `counter` (and any other referenced variables), use the `move` keyword
   |
12 |         children.push(thread::spawn(move || {
   |                                     ^^^^^^^
<snip>
```

The compiler is flat-out telling us we need to make `children` "`mutable`" and we need to `move` `counter` to the thread. Let's explore this further.

1. Start with [`rust-mutability`](https://github.com/pa3ng/rust-memory-basics) to understand how Rust handles mutations of objects.
2. Then take a look at [`rust-memory-basics`](https://github.com/pa3ng/rust-memory-basics) to see how Rust provides memory safety without resorting to garbage collection.
3. Resume from here.

### [`concurrent1.rs`](rust-intermediate-attempts/concurrent1.rs)

Welcome back! You should now understand what `mut` and `move` mean, and you probably know why the example prints `0`. But, let's walk through it. First, let's address the two compile errors above:

```diff
--- concurrent0.rs	2019-05-02 19:31:59.470651148 -0500
+++ concurrent1.rs	2019-05-02 19:33:41.666508644 -0500
@@ -9 +9 @@
-    let children = vec![];
+    let mut children = vec![];
@@ -12 +12 @@
-        children.push(thread::spawn(|| {
+        children.push(thread::spawn(move || {
```

We know this won't compile:

```
$  rustc concurrent1.rs 
error[E0594]: cannot assign to immutable captured outer variable in an `FnOnce` closure `counter`
  --> concurrent1.rs:14:17
   |
14 |                 counter += 1;
   |                 ^^^^^^^^^^^^

error: aborting due to previous error

For more information about this error, try `rustc --explain E0594`.
```

### [`concurrent2.rs`](rust-intermediate-attempts/concurrent2.rs)

`concurrent1`'s compile error is telling us that we succesfully `move`d (really `Copy`ed) our `counter`, but it is not `mut`able. Let's fix that next:

```diff
--- concurrent1.rs	2019-05-02 19:33:41.666508644 -0500
+++ concurrent2.rs	2019-05-02 19:34:14.699456356 -0500
@@ -7 +7 @@
-    let counter: u32 = 0;
+    let mut counter: u32 = 0;
```

This is now the same as our first attempt. We know it compiles, we know what it does.

### [`concurrent3.rs`](rust-intermediate-attempts/concurrent3.rs)

We know we don't want to `Copy`. The easiest way to prevent that is to wrap the `counter` in a `Struct`:

```diff
--- concurrent2.rs	2019-05-02 19:34:14.699456356 -0500
+++ concurrent3.rs	2019-05-07 22:26:04.444538266 -0500
@@ -5,0 +6,4 @@
+struct Counter {
+    val: u32,
+}
+
@@ -7 +11 @@
-    let mut counter: u32 = 0;
+    let mut counter = Counter { val: 0 };
@@ -14 +18 @@
-                counter += 1;
+                counter.val += 1;
@@ -23 +27 @@
-    println!("{}", counter);
+    println!("{}", counter.val);
```

Where does this get us?

```
$ rustc concurrent3.rs
error[E0382]: capture of moved value: `counter`
  --> concurrent3.rs:18:17
   |
16 |         children.push(thread::spawn(move || {
   |                                     ------- value moved (into closure) here
17 |             for _ in 0..NITERATIONS {
18 |                 counter.val += 1;
   |                 ^^^^^^^ value captured here after move
   |
   = note: move occurs because `counter` has type `Counter`, which does not implement the `Copy` trait
<snip>
```

Remember, memory can only have one owner. We can move our new `counter` _once_, but here we're attempting to `move` it multiple times. Fortunately, Rust has an easy solution to this.

### [`concurrent4.rs`](rust-intermediate-attempts/concurrent4.rs)

Rust's [Arc](https://doc.rust-lang.org/std/sync/struct.Arc.html) struct is

    A thread-safe reference-counting pointer. 'Arc' stands for 'Atomically Reference Counted'.

    The type Arc<T> provides shared ownership of a value of type T, allocated in the heap. Invoking clone on Arc produces a new Arc instance, which points to the same value on the heap as the source Arc, while increasing a reference count. When the last Arc pointer to a given value is destroyed, the pointed-to value is also destroyed.

Let's wrap our `counter` in an `Arc`:

```diff
--- concurrent3.rs	2019-05-07 22:26:04.444538266 -0500
+++ concurrent4.rs	2019-05-06 21:55:00.866156855 -0500
@@ -0,0 +1 @@
+use std::sync::Arc;
@@ -11 +12 @@
-    let mut counter = Counter { val: 0 };
+    let mut counter = Arc::new(Counter { val: 0 });
@@ -15,0 +17 @@
+        let counter = Arc::clone(&counter);
```

And compile:

```
$ rustc concurrent4.rs 
error[E0594]: cannot assign to field of immutable binding
  --> concurrent4.rs:20:17
   |
20 |                 counter.val += 1;
   |                 ^^^^^^^^^^^^^^^^ cannot mutably borrow field of immutable binding

warning: variable does not need to be mutable
  --> concurrent4.rs:12:9
   |
12 |     let mut counter = Arc::new(Counter { val: 0 });
   |         ----^^^^^^^
   |         |
   |         help: remove this `mut`
   |
   = note: #[warn(unused_mut)] on by default
<snip>
```

Rust doesn't like our attempt to make `Arc` `mut`able.

### [`concurrent5.rs`](rust-intermediate-attempts/concurrent5.rs)

How do we make our `Counter` struct's `val` `mut`able? We implement a method:

```rust
impl Counter {
    fn increment_counter(&mut self) {
        self.val += 1;
    }
}
```

And then we `counter.increment_counter()`. We'll also remove `mut`:

```diff
--- concurrent4.rs	2019-05-06 21:55:00.866156855 -0500
+++ concurrent5.rs	2019-05-06 21:56:38.095894631 -0500
@@ -10,0 +11,6 @@
+impl Counter {
+    fn increment_counter(&mut self) {
+        self.val += 1;
+    }
+}
+
@@ -12 +18 @@
-    let mut counter = Arc::new(Counter { val: 0 });
+    let counter = Arc::new(Counter { val: 0 });
@@ -20 +26 @@
-                counter.val += 1;
+                counter.increment_counter();
```

Have we fixed it?:

```
$  rustc concurrent5.rs 
error[E0596]: cannot borrow immutable borrowed content as mutable
  --> concurrent5.rs:26:17
   |
26 |                 counter.increment_counter();
   |                 ^^^^^^^ cannot borrow as mutable
<snip>
```

### [`concurrent6.rs`](rust-intermediate-attempts/concurrent6.rs)

Looks like we can't "fake" mutability: we have a function that supports it, but `Arc` doesn't deref our `counter` mutably, so it doesn't matter. Fortunately, the same documentation quoted above -- the next paragraph, in fact -- tells us why the last attempt failed:

    Shared references in Rust disallow mutation by default, and Arc is no exception: you cannot generally obtain a mutable reference to something inside an Arc. If you need to mutate through an Arc, use Mutex, RwLock, or one of the Atomic types.

We'll use [`Mutex`](https://doc.rust-lang.org/std/sync/struct.Mutex.html) here:

```diff
--- concurrent5.rs	2019-05-06 21:56:38.095894631 -0500
+++ concurrent6.rs	2019-05-06 22:00:39.678287698 -0500
@@ -1 +1 @@
-use std::sync::Arc;
+use std::sync::{Arc, Mutex};
@@ -18 +18 @@
-    let counter = Arc::new(Counter { val: 0 });
+    let counter = Arc::new(Mutex::new(Counter { val: 0 }));
@@ -26 +26 @@
-                counter.increment_counter();
+                counter.lock().unwrap().increment_counter();
@@ -35 +35 @@
-    println!("{}", counter.val);
+    println!("{}", counter.lock().unwrap().val);
```

And:

```
$ rustc concurrent6.rs && ./concurrent6
10000000
```

That's it!

## [rust-working-example](rust-working-example/concurrent.rs)

`concurrent6.rs` is the same as working example's `concurrent.rs`. It works, it does what we want. It also highlights another guarantee of Rust:

    If your program compiles, it is thread-safe.

You may have noticed the example isn't particularly fast in contrast to Java's. Rust, in practice, does not recommend our approach. `Arc`'s [Examples](https://doc.rust-lang.org/std/sync/struct.Arc.html?search=#examples) section suggests an alternative in [AtomicUsize](https://doc.rust-lang.org/std/sync/atomic/struct.AtomicUsize.html).

See [`alt_concurrent.rs`](rust-working-example/alt_concurrent.rs) for a better, more performant solution.