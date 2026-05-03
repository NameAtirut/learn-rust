//! Hour 7 — Channels, scoped threads, data parallelism with rayon.
//!
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │ Python ↔ Rust at a glance                                           │
//! ├─────────────────────────────────────────────────────────────────────┤
//! │ queue.Queue                       │ std::sync::mpsc::channel()     │
//! │ q.put(x) / q.get()                │ tx.send(x) / rx.recv()         │
//! │ (no built-in scoped threads)      │ thread::scope(|s| { ... })     │
//! │ multiprocessing.Pool().map(f, xs) │ xs.par_iter().map(f).collect() │
//! │ select on multiple queues         │ crossbeam::select! { ... }     │
//! └─────────────────────────────────────────────────────────────────────┘
//!
//! Two big ideas this hour:
//!
//!   1. "Don't communicate by sharing memory; share memory by
//!      communicating." Channels MOVE ownership of values between threads.
//!      Once you send a value, the sender can't use it anymore — that's
//!      the borrow-checker rule applied across thread boundaries. (In
//!      Python, `queue.Queue` does the same conceptually, but the language
//!      can't enforce that you stop using the object after putting it on
//!      the queue.)
//!
//!   2. For pure CPU-bound data parallelism (map/reduce over a slice),
//!      reach for `rayon`. It's `multiprocessing.Pool().map()` — but
//!      without pickling, without process startup cost, and one method
//!      call away from your existing iterator code (`.par_iter()`).
//!
//! Run with: cargo run --example h7_channels

use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    mpsc_basics();
    fan_out_fan_in();
    scoped_threads();
    rayon_demo();
    crossbeam_channel_demo();
}

// --- mpsc = multi-producer, single-consumer (the std channel) ---
//
// Python:
//   q = queue.Queue()
//   def producer():
//       for i in range(3): q.put(i)
//   threading.Thread(target=producer).start()
//   while True:
//       msg = q.get()  # blocks
fn mpsc_basics() {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        for i in 0..3 {
            tx.send(i).unwrap();             // value is *moved* across threads
            thread::sleep(Duration::from_millis(5));
        }
    });
    // Iterating the receiver blocks until the sender is dropped (channel
    // closed). Rust knows the channel is "done" because all senders went
    // out of scope — Python's Queue has no built-in equivalent (you'd
    // typically send a sentinel like None).
    for msg in rx {
        println!("[mpsc] got {msg}");
    }
}

// --- classic fan-out / fan-in ---
fn fan_out_fan_in() {
    let (tx, rx) = mpsc::channel();
    for i in 0..4 {
        let tx = tx.clone();                 // each producer gets its own Sender
        thread::spawn(move || {
            let result = expensive(i);
            tx.send((i, result)).unwrap();
        });
    }
    drop(tx);                                // drop the original — otherwise
                                              // the receiver loop never ends.
                                              // (Python: send a sentinel.)

    let mut results: Vec<(u64, u64)> = rx.iter().collect();
    results.sort();
    println!("[fan] results = {results:?}");
}

fn expensive(n: u64) -> u64 {
    thread::sleep(Duration::from_millis(10));
    n * n
}

// --- scoped threads: borrow stack data without Arc ---
//
// `thread::scope` (stable since Rust 1.63) is a brilliant API. Threads
// spawned inside it MUST finish before the scope returns, so they can
// borrow data that lives in the parent stack frame. No Arc, no clones.
//
// Python doesn't have anything like this — there's no compile-time
// guarantee that a thread finishes before the function ends.
fn scoped_threads() {
    let data = vec![10, 20, 30, 40, 50, 60, 70, 80];
    let mid = data.len() / 2;
    let (left, right) = data.split_at(mid);

    let (sum_l, sum_r) = thread::scope(|s| {
        let h_l = s.spawn(|| left.iter().sum::<i32>());     // borrows `left`
        let h_r = s.spawn(|| right.iter().sum::<i32>());    // borrows `right`
        (h_l.join().unwrap(), h_r.join().unwrap())
    });
    println!("[scoped] left={sum_l}, right={sum_r}, total={}", sum_l + sum_r);
}

// --- rayon: ".par_iter()" data parallelism ---
//
// Rayon manages a thread pool sized to your CPU count, work-stealing
// scheduler — no async runtime, no channels. Just parallel iterators.
//
// Python parallel: multiprocessing.Pool().map(square, data). Differences:
//   - rayon doesn't pickle anything (one process, multiple threads)
//   - startup is microseconds, not "fork a Python interpreter"
//   - your code change is ONE word: `iter` -> `par_iter`
fn rayon_demo() {
    use rayon::prelude::*;
    let data: Vec<u64> = (1..=1_000_000).collect();

    let t = Instant::now();
    let serial: u64 = data.iter().map(|n| n * n).sum();
    let serial_ms = t.elapsed().as_micros();

    let t = Instant::now();
    let parallel: u64 = data.par_iter().map(|n| n * n).sum();
    let parallel_ms = t.elapsed().as_micros();

    assert_eq!(serial, parallel);
    println!("[rayon] sum of squares serial={serial_ms}µs parallel={parallel_ms}µs");
    // For trivially small per-item work like multiplying two ints, parallel
    // may be SLOWER — overhead dominates. Same trap as Python's
    // multiprocessing. The point is real per-item work (parsing, hashing,
    // image transforms, etc.).
}

// --- crossbeam-channel: a more powerful alternative to mpsc ---
//
// crossbeam offers MPMC (multi-producer MULTI-consumer), bounded channels,
// `select!`, and is generally faster. Use it when std mpsc isn't enough.
//
// Python: there isn't a great direct analog. You'd combine queue.Queue
// with manual logic, or reach for `selectors` + sockets, or asyncio.
fn crossbeam_channel_demo() {
    use crossbeam::channel::{bounded, select};
    let (tx_a, rx_a) = bounded::<&'static str>(1);
    let (tx_b, rx_b) = bounded::<&'static str>(1);

    thread::spawn(move || { thread::sleep(Duration::from_millis(5)); tx_a.send("A").unwrap(); });
    thread::spawn(move || { thread::sleep(Duration::from_millis(2)); tx_b.send("B").unwrap(); });

    // `select!` waits on whichever channel becomes ready FIRST — like Go's
    // select, or like asyncio.wait(..., return_when=FIRST_COMPLETED).
    select! {
        recv(rx_a) -> msg => println!("[crossbeam] first: {:?}", msg.unwrap()),
        recv(rx_b) -> msg => println!("[crossbeam] first: {:?}", msg.unwrap()),
    }
}

// =================== EXERCISES ===================
//
// 1. Pipeline: build a 3-stage pipeline using two channels.
//    Stage A: emit numbers 1..=20.
//    Stage B: square them.
//    Stage C (main): print them.
//    Each stage runs in its own thread. Python parallel: three Threads
//    chained by two queue.Queues.
//
// 2. Convert your parallel-sum exercise from Hour 6 (challenge #1) to use
//    `thread::scope` and slice borrows instead of cloning chunks. Notice
//    you no longer need `move` to transfer ownership of the data.
//
// 3. Take a non-trivial CPU function (e.g. `fn count_primes_up_to(n: u64) -> u64`)
//    and time `data.iter().map(...)` vs `data.par_iter().map(...)` on
//    `(1..=20).collect::<Vec<u64>>()` where each entry is `1_000_000 * i`.
//    This is the kind of speedup Python users buy `multiprocessing` for.
//
// 4. (Stretch) Build a bounded work queue: producer sends N "jobs" on a
//    `crossbeam::channel::bounded(4)` channel; 3 workers receive and
//    process. The bounded channel provides natural backpressure (the
//    producer blocks when the queue is full). Python parallel:
//    queue.Queue(maxsize=4) + multiple worker threads.
