//! Hour 4 — Collections, iterators, error handling.
//!
//! Rust's iterator API is one of its best features. Most loops you'd write
//! in another language become a chain of iterator adapters here, and they
//! compile down to the same machine code as a hand-written loop ("zero-cost
//! abstractions").
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
    let mut v: Vec<i32> = vec![1, 2, 3];      // `vec!` macro
    v.push(4);
    v.extend([5, 6]);
    println!("[vec] {v:?}");

    // Indexing: `v[i]` panics if out of bounds; `v.get(i)` returns Option.
    println!("[vec] v[0]={}, v.get(99)={:?}", v[0], v.get(99));

    // Iteration variants:
    //   for x in &v       -> &i32     (shared borrow)
    //   for x in &mut v   -> &mut i32 (exclusive borrow)
    //   for x in v        -> i32      (consumes v — moves)
    let sum: i32 = (&v).iter().sum();
    println!("[vec] sum = {sum}");
}

fn hashmaps_and_sets() {
    let mut counts: HashMap<&str, i32> = HashMap::new();
    for word in "the quick brown fox the lazy dog the".split_whitespace() {
        *counts.entry(word).or_insert(0) += 1;        // idiomatic counter
    }
    println!("[map] {counts:?}");

    let unique: HashSet<&str> = "a b a c b".split_whitespace().collect();
    println!("[set] {unique:?}");
}

fn iterators() {
    // Iterator pipeline: each step is lazy until consumed by `.collect()`,
    // `.sum()`, `.for_each()`, etc.
    let squares_of_evens: Vec<i32> = (1..=10)
        .filter(|n| n % 2 == 0)
        .map(|n| n * n)
        .collect();
    println!("[iter] squares of evens 1..=10 = {squares_of_evens:?}");

    // `enumerate` for index + value:
    for (i, x) in ["a", "b", "c"].iter().enumerate() {
        println!("[iter] {i}: {x}");
    }

    // `zip` walks two iterators in parallel:
    let names = ["Ada", "Linus", "Grace"];
    let ages  = [205, 54, 117];
    let pairs: Vec<(&&str, &i32)> = names.iter().zip(ages.iter()).collect();
    println!("[iter] zipped = {pairs:?}");

    // `fold` is a generic reducer.
    let product: i32 = (1..=5).fold(1, |acc, n| acc * n);
    println!("[iter] 5! via fold = {product}");

    // Collect into a HashMap directly:
    let lookup: HashMap<&&str, &i32> = names.iter().zip(ages.iter()).collect();
    println!("[iter] lookup[Ada] = {:?}", lookup.get(&"Ada"));

    // `?` works inside iterators with try_fold / collect into Result:
    let parsed: Result<Vec<i32>, _> = ["1", "2", "3"].iter().map(|s| s.parse()).collect();
    println!("[iter] parsed = {parsed:?}");
    let parsed_bad: Result<Vec<i32>, _> = ["1", "x"].iter().map(|s| s.parse()).collect();
    println!("[iter] parsed_bad = {parsed_bad:?}");
}

// --- error handling: idiomatic patterns ---
//
// In libraries, define your own error type (often via `thiserror`).
// In applications, use `anyhow::Result` for "any error" ergonomics.

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
            println!("[err] failed: {e:#}");        // {:#} prints the chain
        }
    }
}

// =================== EXERCISES ===================
//
// 1. Given `let words = ["apple", "pear", "banana", "fig"];`, build a
//    `HashMap<usize, Vec<&str>>` mapping word length to all words of that
//    length. (Hint: `.entry(len).or_insert_with(Vec::new).push(w)`.)
//
// 2. Given `let nums = [3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5];`, find:
//      a) the average (as f64)
//      b) the most frequent value (use a HashMap)
//      c) the unique values, sorted ascending (collect into a BTreeSet, or
//         HashSet then sort)
//
// 3. Implement `fn parse_csv_row(line: &str) -> Result<Vec<i32>>` that
//    splits on ',' and parses each field. Use `collect::<Result<Vec<_>, _>>()`.
//
// 4. (Stretch) Re-implement the word-counter without `entry`, then with it.
//    Notice how borrow checking forces you to think about who owns the
//    HashMap during iteration.
