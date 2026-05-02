//! Hour 2 — Ownership, borrowing, lifetimes.
//!
//! This is THE hour. Rust's whole personality comes from these three rules:
//!
//!   1. Every value has exactly one owner.
//!   2. When the owner goes out of scope, the value is dropped (freed).
//!   3. You can borrow a value: either many shared (`&T`) OR one exclusive
//!      (`&mut T`) reference at a time — never both.
//!
//! Rule 3 is what makes Rust thread-safe by default. Hold onto that — we'll
//! cash it in during Hour 6.
//!
//! Run with: cargo run --example h2_ownership

fn main() {
    moves();
    clones();
    borrows();
    mutable_borrows();
    slices();
    lifetimes_demo();
}

// --- moves: assignment of owned values transfers ownership ---
fn moves() {
    let s1 = String::from("hello");
    let s2 = s1;                // s1 is *moved* into s2. s1 is now invalid.
    // println!("{s1}");        // ❌ value used here after move. Try it.
    println!("[moves] s2 = {s2}");

    // For `Copy` types (integers, bools, char, fixed-size arrays of Copy
    // types, etc.), assignment copies — no move:
    let a = 5;
    let b = a;
    println!("[moves] a={a}, b={b}  (both still valid because i32 is Copy)");
}

// --- clones: explicit deep copy when you actually want it ---
fn clones() {
    let s1 = String::from("hello");
    let s2 = s1.clone();        // explicit, allocates a new String
    println!("[clones] s1={s1}, s2={s2}");
    // Rule of thumb: if `.clone()` shows up a lot, you probably want a
    // borrow instead. Reach for clone only when ownership semantics demand it.
}

// --- borrows: pass a reference instead of moving ---
fn borrows() {
    let s = String::from("borrow me");
    let n = length(&s);         // & creates a shared reference; s still owns the data
    println!("[borrows] len of {s:?} = {n}");
}

fn length(s: &String) -> usize { // takes a &String, doesn't own it
    s.len()
}

// --- mutable borrows: exclusive, exactly one at a time ---
fn mutable_borrows() {
    let mut s = String::from("hi");
    push_excl(&mut s);          // &mut: an exclusive, mutable borrow
    println!("[mut_borrows] {s}");

    // Compiler enforces "many shared XOR one exclusive":
    let r1 = &s;
    let r2 = &s;                // ✅ multiple shared
    println!("{r1} {r2}");
    // After the last use of r1/r2, the compiler "ends" their borrows
    // (this is called Non-Lexical Lifetimes / NLL).
    let r3 = &mut s;            // ✅ now allowed because r1, r2 are no longer used
    r3.push('!');
    println!("[mut_borrows] {r3}");

    // ❌ Try it: have a shared and a mutable borrow alive at the same time:
    // let r1 = &s;
    // let r2 = &mut s;
    // println!("{r1} {r2}"); // error[E0502]: cannot borrow `s` as mutable...
}

fn push_excl(s: &mut String) {
    s.push('!');
}

// --- slices: borrows of pieces of a collection ---
fn slices() {
    let s = String::from("hello world");
    let hello: &str = &s[0..5];   // string slice, borrows part of s
    let world: &str = &s[6..];
    println!("[slices] {hello:?} | {world:?}");

    let arr = [10, 20, 30, 40, 50];
    let mid: &[i32] = &arr[1..4]; // slice of the array
    println!("[slices] mid = {mid:?}");

    // Prefer parameters of type `&str` over `&String`, and `&[T]` over `&Vec<T>` —
    // they accept both owned and borrowed inputs.
    println!("[slices] first_word = {}", first_word("rusty crab"));
}

fn first_word(s: &str) -> &str {
    match s.find(' ') {
        Some(i) => &s[..i],
        None => s,
    }
}

// --- lifetimes: the compiler needs to know how long borrows live ---
//
// A lifetime annotation `'a` is NOT how long a value lives. It's a
// constraint: "the returned reference is valid for at least as long as 'a".
// Most of the time the compiler infers them ("lifetime elision").
//
// Here we MUST annotate, because the return could borrow from either input
// and the compiler refuses to guess:
fn longer<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() >= y.len() { x } else { y }
}

fn lifetimes_demo() {
    let a = String::from("loooong");
    let b = String::from("hi");
    let l = longer(&a, &b);
    println!("[lifetimes] longer = {l}");

    // Why this matters: the borrow checker uses lifetimes to prove no
    // dangling references can exist. The classic dangling pointer bug is
    // *impossible* in safe Rust:
    //
    //   fn dangle() -> &String {
    //       let s = String::from("oops");
    //       &s         // ❌ s is dropped at end of function; ref would dangle
    //   }
}

// =================== EXERCISES ===================
//
// 1. The function below DOES NOT COMPILE. Fix it without using `.clone()`.
//    (Hint: who needs to own `name`? The caller still wants to print it.)
//
//    fn greet(name: String) -> String { format!("hi, {name}") }
//    fn main_ex() {
//        let n = String::from("Ada");
//        let g = greet(n);
//        println!("{n} -> {g}");   // <-- "borrow of moved value: n"
//    }
//
// 2. Write `fn longest_word(s: &str) -> &str` returning the longest
//    whitespace-separated word. (No allocation — return a slice of `s`.)
//
// 3. Implement `fn append_exclaim(s: &mut String)` and call it twice in a
//    row. Then break it: try to call it while a shared `&s` is alive. Read
//    the error message — note the line numbers it points to.
//
// 4. (Stretch) Write a struct that holds a string slice:
//        struct Excerpt<'a> { text: &'a str }
//    and a method `fn first_sentence(&self) -> &str`. The lifetime
//    annotation `'a` is mandatory here — figure out why.
