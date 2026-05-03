//! Hour 6 — Threads, Send/Sync, Arc, Mutex.
//!
//! Now we cash in everything from Hour 2.
//!
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │ The Python user's biggest revelation                                │
//! ├─────────────────────────────────────────────────────────────────────┤
//! │ Python has the GIL: only one thread runs Python bytecode at a time. │
//! │ So `threading` is fine for I/O concurrency but useless for CPU.     │
//! │ For CPU work, you reach for `multiprocessing` and pickle everything.│
//! │                                                                     │
//! │ Rust has NO GIL. Threads run truly in parallel. And — here's the    │
//! │ kicker — DATA RACES ARE A COMPILE ERROR. The borrow checker rule    │
//! │ from Hour 2 ("many shared XOR one exclusive") plus two marker       │
//! │ traits prove this at compile time:                                  │
//! │                                                                     │
//! │   Send: a type can be safely MOVED to another thread.               │
//! │   Sync: a type can be safely SHARED between threads (i.e. &T: Send).│
//! │                                                                     │
//! │ Most types are Send + Sync automatically. The exceptions matter:    │
//! │   Rc<T>     — not Send (non-atomic refcount would race)             │
//! │   Cell<T>   — not Sync (interior mutability sans synchronization)   │
//! │   raw pointers — neither                                            │
//! │ Try to share these across threads → compile error. No surprises.    │
//! └─────────────────────────────────────────────────────────────────────┘
//!
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │ Python ↔ Rust thread cheat sheet                                    │
//! ├─────────────────────────────────────────────────────────────────────┤
//! │ threading.Thread(target=f).start() │ thread::spawn(|| { ... })      │
//! │ t.join()                           │ handle.join()                  │
//! │ threading.Lock() / with lock:      │ Mutex<T> / let g = m.lock()    │
//! │ threading.RLock                    │ (use Mutex; reentrant locks    │
//! │                                    │  are deliberately not in std)  │
//! │ threading.RWLock-ish               │ RwLock<T>                      │
//! │ (sharing — Python: just refer to)  │ Arc<T> (atomic refcount)       │
//! │ multiprocessing for CPU parallelism│ std::thread (true parallel)    │
//! └─────────────────────────────────────────────────────────────────────┘
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
    // Python: t = threading.Thread(target=lambda: ...); t.start(); t.join()
    let handle = thread::spawn(|| {
        for i in 0..3 {
            println!("[spawn]   tick {i}");
            thread::sleep(Duration::from_millis(20));
        }
        42                              // last expression = thread's "return value"
    });
    println!("[spawn] main is doing other work...");
    let result = handle.join().expect("thread panicked");
    println!("[spawn] thread returned {result}");
    // Python's threading API has no clean "thread return value" — you'd
    // typically use a Queue. Rust's join() gives you the value back directly.
}

// --- moving data into a thread ---
//
// A spawned thread might outlive the current scope, so the closure must own
// its captures. That's what `move` does. (`std::thread::scope` lifts this
// restriction — see Hour 7.)
//
// In Python you don't think about this — the GC keeps the object alive
// regardless. In Rust the compiler needs proof, and `move` is that proof.
fn move_into_thread() {
    let v = vec![1, 2, 3];
    // Without `move`, the closure would only borrow v, but the compiler
    // can't prove v lives long enough — error.
    let h = thread::spawn(move || {
        let sum: i32 = v.iter().sum();
        println!("[move] sum in thread = {sum}");
    });
    h.join().unwrap();
    // v is no longer accessible here — it was moved into the thread.
}

// --- sharing mutable state: Arc<Mutex<T>> ---
//
// Python:
//   counter = 0
//   lock = threading.Lock()
//   def worker():
//       global counter
//       for _ in range(1000):
//           with lock: counter += 1
//
// Rust pattern (read this two-layer wrapping carefully):
//   Arc<T>    — Atomic Reference Counted shared pointer (cheap to clone).
//               This is how multiple threads SHARE OWNERSHIP of one value.
//               (You can think of Arc as "Python's default: a reference".)
//   Mutex<T>  — mutual exclusion lock; .lock() returns a guard; the lock
//               is released when the guard is dropped. (Like Python's
//               `with lock:` — but the unlock is automatic at scope end.)
//
// So Arc<Mutex<T>> = "shared, mutex-protected mutable state".
fn shared_state_with_arc_mutex() {
    let counter = Arc::new(Mutex::new(0u64));
    let mut handles = vec![];
    for _ in 0..8 {
        let counter = Arc::clone(&counter);     // bumps refcount, doesn't deep-copy
        handles.push(thread::spawn(move || {
            for _ in 0..1_000 {
                let mut g = counter.lock().unwrap();   // acquire lock
                *g += 1;
                // lock released here when `g` is dropped (end of scope) —
                // no `with` block needed, no risk of forgetting to unlock.
            }
        }));
    }
    for h in handles { h.join().unwrap(); }
    println!("[arc-mutex] final = {}", *counter.lock().unwrap());
    // Expect: 8000. If you tried to share `counter` without Mutex, the
    // program WOULDN'T COMPILE. Python won't stop you from doing the unsafe
    // thing — it'll just give you wrong answers (or weirder, in the GIL
    // era, sometimes correct ones for the wrong reasons).
}

// --- RwLock when reads vastly outnumber writes ---
//
// Python: threading.RLock won't help you here; you'd typically just use
// Lock. RWLock implementations exist in third-party libs.
fn rwlock_for_read_heavy() {
    let config = Arc::new(RwLock::new(String::from("v1")));
    let mut handles = vec![];

    // Many readers in parallel
    for id in 0..4 {
        let config = Arc::clone(&config);
        handles.push(thread::spawn(move || {
            let g = config.read().unwrap();           // many readers OK
            println!("[rwlock] reader {id} sees {}", *g);
        }));
    }
    // One writer (gets exclusive access)
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
// Same gotcha as Python: hold lock A, try to take lock B, while another
// thread holds B and is trying to take A → deadlock. Rule of thumb: always
// acquire locks in a consistent order. Better: keep critical sections
// short, and never hold one lock while calling code that could acquire
// another.
fn deadlock_demo_safe() {
    let a = Arc::new(Mutex::new(0));
    let b = Arc::new(Mutex::new(0));

    // We *don't* deadlock here because both threads acquire a, then b.
    // (Reverse the order in one thread for fun → almost certain deadlock.)
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
// 1. Parallel sum: split `(1..=1_000_000u64)` into N chunks, sum each in
//    its own thread, sum the partials in main. Verify the answer is correct.
//    No Mutex needed — each thread owns its chunk.
//    Python equivalent: multiprocessing.Pool().map(sum, chunks). Notice that
//    Rust does this with threads (no pickling) because there's no GIL.
//
// 2. Worker-pool sketch: spawn 4 threads that each pull from a shared
//    `Arc<Mutex<Vec<Job>>>` (a Job can be a closure or just a number).
//    When the Vec is empty, they exit. Why is a CHANNEL a better fit here
//    than a shared Mutex? (Spoiler: Hour 7.)
//
// 3. Provoke a compile error: try to spawn a thread that captures an
//    `Rc<i32>`. Read the error — note it mentions `Send`. Replace `Rc`
//    with `Arc` to fix. THIS is the kind of bug Python silently lets you
//    make and then debug for hours.
//
// 4. (Stretch) Implement an `AtomicCounter` using
//    `std::sync::atomic::AtomicU64` (lock-free, no Mutex). Time it against
//    the Mutex version on 8 threads × 100_000 increments using
//    `std::time::Instant`.
