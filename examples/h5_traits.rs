//! Hour 5 — Traits, generics, closures.
//!
//! Traits are Rust's interface system. Generics are how you write code once
//! that works for many types. Closures are anonymous functions that can
//! capture their environment. These three together are how you write
//! abstract, fast Rust.
//!
//! Run with: cargo run --example h5_traits

use std::fmt::Display;

fn main() {
    traits_basics();
    generics_and_bounds();
    trait_objects();
    closures();
    iterators_use_all_three();
}

// --- defining a trait ---
trait Greet {
    /// Required: implementors must provide this.
    fn name(&self) -> String;

    /// Default method: implementors get this for free, can override.
    fn greet(&self) -> String {
        format!("Hello, {}!", self.name())
    }
}

struct Cat { nickname: String }
struct Dog;

impl Greet for Cat {
    fn name(&self) -> String { self.nickname.clone() }
    fn greet(&self) -> String { format!("*{} ignores you*", self.nickname) }
}

impl Greet for Dog {
    fn name(&self) -> String { "dog".into() }
    // uses the default greet()
}

fn traits_basics() {
    let c = Cat { nickname: "Mochi".into() };
    let d = Dog;
    println!("[trait] {}", c.greet());
    println!("[trait] {}", d.greet());
}

// --- generics: write once, run for many ---
//
// `T: Display` is a "trait bound" — T must implement Display.
fn announce<T: Display>(x: T) {
    println!("[generic] announcing: {x}");
}

// Multiple bounds with `+`. The `where` clause is a clearer alternative.
fn print_both<T, U>(a: T, b: U)
where
    T: Display,
    U: Display,
{
    println!("[generic] {a} and {b}");
}

fn generics_and_bounds() {
    announce(42);
    announce("hi");
    announce(3.14);
    print_both("answer", 42);
}

// --- trait objects: dynamic dispatch when you need heterogeneous collections ---
//
// `&dyn Greet` / `Box<dyn Greet>` is a "fat pointer" (data + vtable).
// Use it when the concrete type isn't known at compile time.
fn trait_objects() {
    let pets: Vec<Box<dyn Greet>> = vec![
        Box::new(Cat { nickname: "Whiskers".into() }),
        Box::new(Dog),
    ];
    for p in &pets {
        println!("[dyn] {}", p.greet());
    }
    // Generics use static dispatch (monomorphization → faster, larger binary).
    // Trait objects use dynamic dispatch (one binary, slight runtime cost).
    // Reach for trait objects mainly when you need a heterogeneous collection.
}

// --- closures: anonymous functions with environment capture ---
fn closures() {
    let add = |a: i32, b: i32| a + b;          // explicit type
    let inc = |x| x + 1;                        // type inferred from use
    println!("[closure] add(2,3)={}, inc(10)={}", add(2, 3), inc(10));

    // Captures: by reference by default, by value with `move`.
    let prefix = String::from(">> ");
    let say = |msg: &str| println!("{prefix}{msg}");   // borrows prefix
    say("hello");
    say("world");

    // `move` is essential when sending closures to other threads (next hour).
    let owned_prefix = String::from("[owned] ");
    let owning_closure = move |msg: &str| println!("{owned_prefix}{msg}");
    owning_closure("inside");
    // owned_prefix is gone here — moved into the closure.

    // Three closure traits, in order from most to least restrictive call site:
    //   FnOnce — can be called once (consumes captured values)
    //   FnMut  — can be called multiple times, may mutate captures
    //   Fn     — can be called many times without mutation
    // The compiler picks the most permissive trait that fits your closure.
    let mut counter = 0;
    let mut tick = || { counter += 1; counter };       // FnMut
    println!("[closure] tick={}, tick={}, tick={}", tick(), tick(), tick());
}

// --- bringing it together: iterators are traits + generics + closures ---
fn iterators_use_all_three() {
    // `Iterator` is a trait. `.map`, `.filter` take closures. Each adapter
    // is a generic struct. Yet it reads like Python.
    let total: i32 = (1..=100)
        .filter(|n| n % 3 == 0 || n % 5 == 0)
        .sum();
    println!("[combo] Project Euler 1: sum of multiples of 3 or 5 below 100 = {total}");
}

// =================== EXERCISES ===================
//
// 1. Define `trait Area { fn area(&self) -> f64; }` and implement it for a
//    `Circle` and `Square`. Write `fn total_area(shapes: &[Box<dyn Area>]) -> f64`.
//
// 2. Write a generic `fn max_by_key<T, K: Ord>(items: &[T], key: impl Fn(&T) -> K) -> Option<&T>`.
//    Test it with `max_by_key(&["abc","de","fghi"], |s| s.len())`.
//
// 3. Make `Cat` printable with `Display` (impl `std::fmt::Display`) and use
//    it in `announce`.
//
// 4. (Stretch) Write a `fn make_counter() -> impl FnMut() -> u32` that
//    returns a closure incrementing a private counter on each call. Note:
//    `impl FnMut` in return position is the modern, ergonomic syntax.
