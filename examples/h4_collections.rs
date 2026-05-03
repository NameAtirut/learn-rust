//! Hour 4 — Collections, iterators, error handling.
//!
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │ Python ↔ Rust collections cheat sheet                               │
//! ├─────────────────────────────────────────────────────────────────────┤
//! │ Python                  │ Rust                                      │
//! │ ──────                  │ ────                                      │
//! │ list                    │ Vec<T>                                    │
//! │ tuple                   │ (T1, T2, ...) (fixed) or [T; N] (array)   │
//! │ dict                    │ HashMap<K, V>                             │
//! │ set                     │ HashSet<T>                                │
//! │ collections.deque       │ VecDeque<T>                               │
//! │ heapq                   │ BinaryHeap<T>                             │
//! │ sorted dict (3.7+ ins.) │ BTreeMap<K, V> (true sorted by key)       │
//! │ [x*x for x in xs if ..] │ xs.iter().filter(..).map(..).collect()    │
//! │ sum(xs)                 │ xs.iter().sum()                           │
//! │ enumerate(xs)           │ xs.iter().enumerate()                     │
//! │ zip(xs, ys)             │ xs.iter().zip(ys.iter())                  │
//! │ functools.reduce        │ xs.iter().fold(init, |acc,x| ...)         │
//! └─────────────────────────────────────────────────────────────────────┘
//!
//! Iterators in Rust are LAZY (like Python generators). Adapters like
//! .map/.filter don't do anything until you "consume" them with .collect()
//! / .sum() / for. They also compile down to the same machine code as a
//! hand-written loop ("zero-cost abstractions"), unlike Python where each
//! map/filter is a real function call.
//!
//! Run with: cargo run --example h4_collections

use std::collections::{HashMap, HashSet};

fn main() {
    vectors();
    hashmaps_and_sets();
    iterators();
    error_handling_with_anyhow();
}

fn vectors() {
    // Python: v = [1, 2, 3]; v.append(4); v.extend([5, 6])
    let mut v: Vec<i32> = vec![1, 2, 3];      // `vec!` macro = list literal
    v.push(4);
    v.extend([5, 6]);
    println!("[vec] {v:?}");

    // Indexing:
    //   v[i]      -> panics on out of range (like Python's IndexError)
    //   v.get(i)  -> returns Option<&T>      (like dict.get with a default)
    println!("[vec] v[0]={}, v.get(99)={:?}", v[0], v.get(99));

    // Iteration variants. This is something Python doesn't have to think
    // about because of GC, but in Rust you'll pick one consciously:
    //   for x in &v       -> &i32     (shared borrow — read only)
    //   for x in &mut v   -> &mut i32 (exclusive borrow — read+write)
    //   for x in v        -> i32      (consumes v — moves out)
    let sum: i32 = (&v).iter().sum();
    println!("[vec] sum = {sum}");
}

fn hashmaps_and_sets() {
    // Python: counts = {}
    //         for word in s.split(): counts[word] = counts.get(word, 0) + 1
    // Or:     counts = collections.Counter(s.split())
    let mut counts: HashMap<&str, i32> = HashMap::new();
    for word in "the quick brown fox the lazy dog the".split_whitespace() {
        // `entry` is the idiomatic counter-pattern. Closest Python parallel:
        // `counts[word] = counts.get(word, 0) + 1`, but in one shot.
        *counts.entry(word).or_insert(0) += 1;
    }
    println!("[map] {counts:?}");

    // Python: unique = set("a b a c b".split())
    let unique: HashSet<&str> = "a b a c b".split_whitespace().collect();
    println!("[set] {unique:?}");
}

fn iterators() {
    // Python: [n*n for n in range(1, 11) if n % 2 == 0]
    let squares_of_evens: Vec<i32> = (1..=10)
        .filter(|n| n % 2 == 0)
        .map(|n| n * n)
        .collect();
    println!("[iter] squares of evens 1..=10 = {squares_of_evens:?}");

    // Python: for i, x in enumerate(["a", "b", "c"]):
    for (i, x) in ["a", "b", "c"].iter().enumerate() {
        println!("[iter] {i}: {x}");
    }

    // Python: list(zip(names, ages))
    let names = ["Ada", "Linus", "Grace"];
    let ages  = [205, 54, 117];
    let pairs: Vec<(&&str, &i32)> = names.iter().zip(ages.iter()).collect();
    println!("[iter] zipped = {pairs:?}");

    // Python: functools.reduce(lambda acc, n: acc * n, range(1, 6), 1)
    let product: i32 = (1..=5).fold(1, |acc, n| acc * n);
    println!("[iter] 5! via fold = {product}");

    // Python: dict(zip(names, ages))
    let lookup: HashMap<&&str, &i32> = names.iter().zip(ages.iter()).collect();
    println!("[iter] lookup[Ada] = {:?}", lookup.get(&"Ada"));

    // Cool trick: collecting into Result<Vec<_>, _> short-circuits on the
    // first error. Python equivalent would be a try/except inside a loop.
    let parsed: Result<Vec<i32>, _> = ["1", "2", "3"].iter().map(|s| s.parse()).collect();
    println!("[iter] parsed = {parsed:?}");
    let parsed_bad: Result<Vec<i32>, _> = ["1", "x"].iter().map(|s| s.parse()).collect();
    println!("[iter] parsed_bad = {parsed_bad:?}");
}

// --- error handling: idiomatic patterns ---
//
// Python style:
//   try:
//       text = open(path).read()
//       n = int(text.splitlines()[0])
//   except FileNotFoundError as e:
//       raise RuntimeError(f"reading {path}") from e
//
// Rust style: `Result` everywhere, `?` to propagate, `anyhow::Context` to
// add the "while doing X" message that you'd write in `from e`. This pattern
// composes really well — every step says what it was trying to do.

use anyhow::{Context, Result};

fn read_first_int(path: &str) -> Result<i64> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("reading {path}"))?;
    let first = text.lines().next().context("file is empty")?;
    let n: i64 = first.trim().parse().with_context(|| format!("parsing {first:?}"))?;
    Ok(n)
}

fn error_handling_with_anyhow() {
    // Will return an Err — we show how the context chain reads.
    match read_first_int("/no/such/file") {
        Ok(n) => println!("[err] got {n}"),
        Err(e) => {
            // {:#} prints the chain — like Python's "...The above exception
            // was the direct cause of the following exception".
            println!("[err] failed: {e:#}");
        }
    }
}

// =================== EXERCISES ===================
//
// 1. Given `let words = ["apple", "pear", "banana", "fig"];`, build a
//    `HashMap<usize, Vec<&str>>` mapping word length to all words of that
//    length.
//    Python: groups = {}
//            for w in words: groups.setdefault(len(w), []).append(w)
//    Rust idiom: `.entry(len).or_insert_with(Vec::new).push(w)`.
//
// 2. Given `let nums = [3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5];`, compute:
//      a) the average (as f64)               — Python: sum(nums)/len(nums)
//      b) the most frequent value             — Python: Counter(nums).most_common(1)
//      c) unique values, sorted ascending     — Python: sorted(set(nums))
//
// 3. Implement `fn parse_csv_row(line: &str) -> Result<Vec<i32>>` that
//    splits on ',' and parses each field. Use `collect::<Result<Vec<_>, _>>()`
//    to short-circuit on the first parse error.
//
// 4. (Stretch) Re-implement the word-counter without `entry`, then with it.
//    Notice how borrow checking forces you to think about who owns the
//    HashMap during iteration — something Python lets you ignore (until it
//    raises `RuntimeError: dictionary changed size during iteration`).
