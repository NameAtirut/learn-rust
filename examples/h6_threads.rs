//! Hour 6 — Threads, Send/Sync, Arc, Mutex.
//!
//! Now we cash in the borrow checker. Rust's promise: data races are a
//! compile error, not a runtime bug. The compiler enforces this through two
//! marker traits:
//!
//!   - `Send`: a type can be safely *moved* to another thread.
//!   - `Sync`: a type can be safely *shared* between threads (i.e. &T is Send).
//!
//! Most types are Send + Sync automatically. The exceptions are intentional:
//! `Rc<T>` is not Send (its non-atomic refcount would race), `Cell<T>` is
//! not Sync (interior mutability without synchronization), raw pointers are
//! neither. The compiler stops you from sharing them across threads.
//!
//! Run with: cargo run --example h6_threads

use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Duration;

fn main() {
    spawn_and_join();
    move_into_thread();
    shared_state_with_arc_mutex();
    rwlock_for_read_heavy();
    deadlock_demo_safe();
}

// --- spawn / join ---
fn spawn_and_join() {
    let handle = thread::spawn(|| {
        // runs on a new OS thread
        for i in 0..3 {
            println!("[spawn]   tick {i}");
            thread::sleep(Duration::from_millis(20));
        }
        42                              // last expression is the return value
    });
    println!("[spawn] main is doing other work...");
    let result = handle.join().expect("thread panicked");
    println!("[spawn] thread returned {result}");
}

// --- moving data into a thread ---
//
// A spawned thread might outlive the current scope, so the closure must own
// its captures. That's what `move` does. (`std::thread::scope` lifts this
// restriction — see `h7_channels.rs`.)
fn move_into_thread() {
    let v = vec![1, 2, 3];
    // Without `move`, the closure would only borrow v, but the compiler can't
    // prove v lives long enough — error.
    let h = thread::spawn(move || {
        let sum: i32 = v.iter().sum();
        println!("[move] sum in thread = {sum}");
    });
    h.join().unwrap();
    // v is no longer accessible here — it was moved.
}

// --- sharing mutable state: Arc<Mutex<T>> ---
//
// Arc<T>      = Atomic Reference Counted shared pointer (clones are cheap).
// Mutex<T>    = mutual exclusion lock; .lock() returns a guard; unlocks on drop.
//
// Pattern: wrap mutable state in Mutex, share it via Arc.
fn shared_state_with_arc_mutex() {
    let counter = Arc::new(Mutex::new(0u64));
    let mut handles = vec![];
    for _ in 0..8 {
        let counter = Arc::clone(&counter);     // bumps refcount, doesn't deep-copy
        handles.push(thread::spawn(move || {
            for _ in 0..1_000 {
                let mut g = counter.lock().unwrap();   // acquire lock
                *g += 1;
                // lock released here when `g` is dropped (end of scope)
            }
        }));
    }
    for h in handles { h.join().unwrap(); }
    println!("[arc-mutex] final = {}", *counter.lock().unwrap());
    // Expect: 8000. If you remove the Mutex and use raw shared mutability,
    // the program won't compile — that's the data-race-freedom guarantee.
}

// --- RwLock when reads vastly outnumber writes ---
fn rwlock_for_read_heavy() {
    let config = Arc::new(RwLock::new(String::from("v1")));
    let mut handles = vec![];

    // Many readers
    for id in 0..4 {
        let config = Arc::clone(&config);
        handles.push(thread::spawn(move || {
            let g = config.read().unwrap();           // many readers OK
            println!("[rwlock] reader {id} sees {}", *g);
        }));
    }
    // One writer
    {
        let config = Arc::clone(&config);
        handles.push(thread::spawn(move || {
            thread::sleep(Duration::from_millis(5));
            let mut g = config.write().unwrap();      // exclusive
            *g = String::from("v2");
            println!("[rwlock] writer updated to {}", *g);
        }));
    }
    for h in handles { h.join().unwrap(); }
}

// --- deadlocks are still possible — Rust prevents data races, not logic bugs ---
//
// Rule of thumb: always acquire locks in a consistent order. Better: keep
// critical sections short, and never hold one lock while calling code that
// could acquire another.
fn deadlock_demo_safe() {
    // We *don't* deadlock here because we acquire a, then b, in BOTH threads.
    // (Reverse the order in one thread for fun → almost certain deadlock.)
    let a = Arc::new(Mutex::new(0));
    let b = Arc::new(Mutex::new(0));

    let h1 = {
        let (a, b) = (Arc::clone(&a), Arc::clone(&b));
        thread::spawn(move || {
            let _ga = a.lock().unwrap();
            thread::sleep(Duration::from_millis(1));
            let _gb = b.lock().unwrap();
            println!("[deadlock-demo] thread 1 acquired a then b");
        })
    };
    let h2 = {
        let (a, b) = (Arc::clone(&a), Arc::clone(&b));
        thread::spawn(move || {
            let _ga = a.lock().unwrap();
            let _gb = b.lock().unwrap();
            println!("[deadlock-demo] thread 2 acquired a then b");
        })
    };
    h1.join().unwrap();
    h2.join().unwrap();
}

// =================== EXERCISES ===================
//
// 1. Parallel sum: split `(1..=1_000_000u64)` into N chunks, sum each in its
//    own thread, sum the partials in main. Verify against `(1..=N).sum()`.
//    No Mutex needed — each thread owns its chunk.
//
// 2. Worker pool sketch: spawn 4 threads that each pull from a shared
//    `Arc<Mutex<Vec<Job>>>` (a Job is a closure or just a number). When the
//    Vec is empty, they exit. Why is a channel a better fit here? (Spoiler:
//    Hour 7.)
//
// 3. Provoke a compile error: try to spawn a thread that captures an `Rc<i32>`.
//    Read the error — note it mentions `Send`. Replace `Rc` with `Arc` to fix.
//
// 4. (Stretch) Implement an `AtomicCounter` using
//    `std::sync::atomic::AtomicU64` (no Mutex, lock-free). Compare timing
//    with the Mutex version on 8 threads × 100_000 increments using
//    `std::time::Instant`.
