//! Hour 5 — Traits, generics, closures.
//!
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │ Python ↔ Rust at a glance                                           │
//! ├─────────────────────────────────────────────────────────────────────┤
//! │ Python                              │ Rust                          │
//! │ ──────                              │ ────                          │
//! │ class Greet(Protocol):              │ trait Greet { fn name() ...; }│
//! │     def name(self) -> str: ...      │                               │
//! │ class Cat: def name(self): ...      │ impl Greet for Cat { ... }    │
//! │ def f(g: Greet): ...                │ fn f<T: Greet>(g: T) { ... }  │
//! │ TypeVar / generics                  │ generic <T> with bounds       │
//! │ lambda x: x + 1                     │ |x| x + 1                     │
//! │ functools.partial / closures        │ closures with `move`          │
//! │ duck typing / isinstance            │ trait bounds checked at compile │
//! │ ABCs / @abstractmethod              │ trait methods (required)      │
//! │ Default methods on ABCs             │ trait methods with default body│
//! └─────────────────────────────────────────────────────────────────────┘
//!
//! Mental model: a `trait` is a Python `Protocol` that's CHECKED AT COMPILE
//! TIME and is FAST (zero-cost when used generically). It's how Rust does
//! abstraction — interfaces, not inheritance.
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
//
// Python:
//   class Greet(Protocol):
//       def name(self) -> str: ...
//       def greet(self) -> str:
//           return f"Hello, {self.name()}!"   # default
trait Greet {
    /// Required: implementors must provide this. (Like @abstractmethod.)
    fn name(&self) -> String;

    /// Default method: implementors get this for free, can override.
    fn greet(&self) -> String {
        format!("Hello, {}!", self.name())
    }
}

struct Cat { nickname: String }
struct Dog;

// `impl Greet for Cat` is the equivalent of declaring `class Cat(Greet)` —
// but you can do it for types you don't own (i.e. types from other crates).
// Python has no real equivalent except monkey-patching.
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

// --- generics: write once, run for many types ---
//
// Python:
//   def announce(x: T) -> None:
//       print(f"announcing: {x}")            # works for anything with __str__
//
// Rust requires you to declare what `T` must support — the "trait bound":
// `T: Display`. This is what mypy `Protocol` is approximating.
fn announce<T: Display>(x: T) {
    println!("[generic] announcing: {x}");
}

// Multiple bounds with `+`. The `where` clause is a clearer alternative
// when the list gets long.
fn print_both<T, U>(a: T, b: U)
where
    T: Display,
    U: Display,
{
    println!("[generic] {a} and {b}");
}

fn generics_and_bounds() {
    // Each call site generates a specialized version (monomorphization) —
    // it's like Python templating + JIT-style specialization, but at compile
    // time. Result: zero runtime cost, larger binary.
    announce(42);
    announce("hi");
    announce(3.14);
    print_both("answer", 42);
}

// --- trait objects: dynamic dispatch when you need heterogeneous collections ---
//
// Python: a list of Greet objects "just works" — every method call is a
// dynamic lookup anyway.
// Rust: you need to opt in with `dyn Greet`. `Box<dyn Greet>` is a "fat
// pointer" — pointer to data + pointer to vtable. Runtime cost: one extra
// indirection per method call.
fn trait_objects() {
    let pets: Vec<Box<dyn Greet>> = vec![
        Box::new(Cat { nickname: "Whiskers".into() }),
        Box::new(Dog),
    ];
    for p in &pets {
        println!("[dyn] {}", p.greet());
    }
    // Rule of thumb: prefer generics (`<T: Trait>`) by default. Reach for
    // `dyn Trait` mainly when you need a heterogeneous collection — the
    // exact case Python users naturally reach for.
}

// --- closures: anonymous functions with environment capture ---
//
// Python:
//   add = lambda a, b: a + b
//   prefix = ">> "
//   say = lambda msg: print(prefix + msg)        # captures `prefix`
fn closures() {
    let add = |a: i32, b: i32| a + b;          // explicit type
    let inc = |x| x + 1;                        // type inferred from usage
    println!("[closure] add(2,3)={}, inc(10)={}", add(2, 3), inc(10));

    // Captures: by reference by default, by value with `move`.
    let prefix = String::from(">> ");
    let say = |msg: &str| println!("{prefix}{msg}");   // borrows prefix
    say("hello");
    say("world");

    // `move` is the keyword you'll need for sending closures to other
    // threads (next hour). Closest Python parallel: passing partial
    // application via `functools.partial(f, x)` — but Rust forces explicit
    // ownership transfer.
    let owned_prefix = String::from("[owned] ");
    let owning_closure = move |msg: &str| println!("{owned_prefix}{msg}");
    owning_closure("inside");
    // owned_prefix is gone here — moved into the closure.

    // Three closure traits (compiler picks the most permissive that fits):
    //   FnOnce — can be called once (consumes captured values)
    //   FnMut  — can be called multiple times, may mutate captures
    //   Fn     — can be called many times without mutation
    // Python doesn't distinguish — every closure can do everything. Rust's
    // distinction matters because it tells callers what they can do with
    // your closure (call once, repeatedly, in parallel, etc.).
    let mut counter = 0;
    let mut tick = || { counter += 1; counter };       // FnMut
    println!("[closure] tick={}, tick={}, tick={}", tick(), tick(), tick());
}

// --- bringing it together: iterators are traits + generics + closures ---
fn iterators_use_all_three() {
    // `Iterator` is a trait. `.map`, `.filter` take closures. Each adapter
    // is a generic struct. Yet it reads like Python (and compiles to a
    // tight loop with no allocations).
    //
    // Python: sum(n for n in range(1,100) if n%3==0 or n%5==0)
    let total: i32 = (1..=100)
        .filter(|n| n % 3 == 0 || n % 5 == 0)
        .sum();
    println!("[combo] Project Euler 1: sum of multiples of 3 or 5 below 100 = {total}");
}

// =================== EXERCISES ===================
//
// 1. Define `trait Area { fn area(&self) -> f64; }` and implement it for
//    `Circle` and `Square`. Write `fn total_area(shapes: &[Box<dyn Area>]) -> f64`.
//    Python parallel: an ABC `Area` and a function summing `s.area()` for
//    each `s` in a list of mixed types.
//
// 2. Write a generic `fn max_by_key<T, K: Ord>(items: &[T], key: impl Fn(&T) -> K) -> Option<&T>`.
//    Test with `max_by_key(&["abc","de","fghi"], |s| s.len())`.
//    Python: max(items, key=lambda s: len(s))
//
// 3. Make `Cat` printable with `Display` (impl `std::fmt::Display`) and
//    pass it to `announce`. Display is the equivalent of __str__; Debug is
//    the equivalent of __repr__.
//
// 4. (Stretch) Write a `fn make_counter() -> impl FnMut() -> u32` that
//    returns a closure incrementing a private counter on each call.
//    Python: def make_counter(): n=[0]; def inc(): n[0]+=1; return n[0]; return inc
//    Note: `impl FnMut` in return position is the modern way to say "I'm
//    returning some closure, you don't need to know exactly which type".
