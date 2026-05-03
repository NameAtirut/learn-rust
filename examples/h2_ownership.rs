//! Hour 2 — Ownership, borrowing, lifetimes.
//!
//! THIS IS THE HOUR. If Rust feels alien, it's because of this hour.
//! Take it slow.
//!
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │ Python ↔ Rust mental model                                          │
//! ├─────────────────────────────────────────────────────────────────────┤
//! │ Python: every variable is a NAME bound to an object on the heap.    │
//! │   Multiple names can point to the same object (`a = b = []`).       │
//! │   The garbage collector frees objects when no names refer to them.  │
//! │   Mutation through any name is visible through every other name.    │
//! │                                                                     │
//! │ Rust: every value has exactly ONE owner.                            │
//! │   Assignment MOVES ownership (the old name is invalidated).         │
//! │   You can BORROW (`&value`) to give temporary access without moving.│
//! │   Borrows obey: many shared (`&T`) OR one exclusive (`&mut T`).     │
//! │   When the owner goes out of scope, the value is freed. No GC.      │
//! └─────────────────────────────────────────────────────────────────────┘
//!
//! The "many shared XOR one exclusive" rule is the secret to Rust's whole
//! safety story. It will save you from data races in Hour 6 — and it's
//! also what catches "modify a list while iterating it" bugs at compile
//! time. (In Python: `RuntimeError: dictionary changed size during
//! iteration`. In Rust: a compile error.)
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
    // Python: a = "hello"; b = a; print(a)  # works fine — both point to same str
    // Rust  : a String OWNS its heap buffer. `let s2 = s1` MOVES that
    //         ownership; s1 is no longer usable.
    let s1 = String::from("hello");
    let s2 = s1;                // s1 is *moved* into s2. s1 is now invalid.
    // println!("{s1}");        // ❌ value used here after move. Try it.
    println!("[moves] s2 = {s2}");

    // For `Copy` types (small, stack-only: integers, bools, char, fixed-size
    // arrays of Copy types, etc.), assignment COPIES — no move:
    let a = 5;
    let b = a;
    println!("[moves] a={a}, b={b}  (both still valid because i32 is Copy)");

    // Mental model: think of Strings/Vecs/HashMaps as having a "deed" that
    // says who owns the underlying buffer. Only one deed exists. When you
    // assign, the deed changes hands. With i32 etc., they're so small that
    // it's cheaper to just copy them — so Rust does, automatically.
}

// --- clones: explicit deep copy when you actually want it ---
fn clones() {
    // Python: import copy; b = copy.deepcopy(a)
    // Rust  : a.clone()  — explicit, allocates a new String
    let s1 = String::from("hello");
    let s2 = s1.clone();
    println!("[clones] s1={s1}, s2={s2}");
    // Rule of thumb: if you're typing `.clone()` a lot, you probably want a
    // borrow instead. Reach for clone only when ownership semantics demand it
    // (e.g. you genuinely need two independent copies).
}

// --- borrows: pass a reference instead of moving ---
fn borrows() {
    // Python (sort of): you've always passed references implicitly. In Rust
    // you make it explicit with `&`. Think of `&s` as "lend out s, don't
    // give it away".
    let s = String::from("borrow me");
    let n = length(&s);         // & creates a shared reference; s still owns the data
    println!("[borrows] len of {s:?} = {n}");
    // After length() returns, the borrow ends and we can use s again.
}

fn length(s: &String) -> usize { // takes a &String, doesn't own it
    s.len()
}

// --- mutable borrows: exclusive, exactly one at a time ---
fn mutable_borrows() {
    // Python lets you mutate a shared list from anywhere — that's the
    // source of "I changed it from one place and broke another" bugs.
    // Rust's rule: while a mutable borrow exists, NO other access (read
    // or write) is allowed. The compiler enforces this.
    let mut s = String::from("hi");
    push_excl(&mut s);          // &mut: an exclusive, mutable borrow
    println!("[mut_borrows] {s}");

    // Compiler enforces "many shared XOR one exclusive":
    let r1 = &s;
    let r2 = &s;                // ✅ multiple shared refs are fine
    println!("{r1} {r2}");
    // After the last use of r1/r2, the compiler "ends" their borrows
    // (this is "Non-Lexical Lifetimes" / NLL — borrows live as long as
    // they're used, not until the end of the scope).
    let r3 = &mut s;            // ✅ now allowed because r1, r2 aren't used again
    r3.push('!');
    println!("[mut_borrows] {r3}");

    // ❌ Try uncommenting these to see the error:
    // let r1 = &s;
    // let r2 = &mut s;
    // println!("{r1} {r2}"); // error[E0502]: cannot borrow `s` as mutable...
    //
    // This is the rule that makes Rust thread-safe. Read it again. We'll
    // cash this in during Hour 6.
}

fn push_excl(s: &mut String) {
    s.push('!');
}

// --- slices: borrows of pieces of a collection ---
fn slices() {
    // Python: s[0:5] — gives a NEW string (a copy)
    // Rust  : &s[0..5] — gives a &str slice that BORROWS from s (no copy)
    let s = String::from("hello world");
    let hello: &str = &s[0..5];
    let world: &str = &s[6..];
    println!("[slices] {hello:?} | {world:?}");

    let arr = [10, 20, 30, 40, 50];
    let mid: &[i32] = &arr[1..4];   // slice of an array (cf Python arr[1:4])
    println!("[slices] mid = {mid:?}");

    // Python: def first_word(s): return s.split()[0]
    // Rust idiom: prefer parameters of type `&str` over `&String`, and
    // `&[T]` over `&Vec<T>` — they accept both owned and borrowed inputs.
    println!("[slices] first_word = {}", first_word("rusty crab"));
}

fn first_word(s: &str) -> &str {
    match s.find(' ') {
        Some(i) => &s[..i],
        None => s,
    }
}

// --- lifetimes: how long a borrow is allowed to exist ---
//
// This is where Python intuition gives out. There's no Python analog —
// Python's GC means references can live as long as you want.
//
// In Rust, every borrow has a lifetime: a span during which the borrow is
// valid. The compiler usually infers them; sometimes you must spell them
// out. A lifetime annotation `'a` is NOT how long a value lives — it's a
// constraint: "the returned reference is valid for at least as long as 'a".
//
// Below: the function might return either x or y. The compiler refuses to
// guess which, so we annotate that the result borrows for the SHORTER of
// the two input lifetimes ('a here is "whichever is shorter"):
fn longer<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() >= y.len() { x } else { y }
}

fn lifetimes_demo() {
    let a = String::from("loooong");
    let b = String::from("hi");
    let l = longer(&a, &b);
    println!("[lifetimes] longer = {l}");

    // Why this matters: the borrow checker uses lifetimes to prove no
    // dangling references can exist. The classic dangling-pointer bug —
    // returning a reference to something local — is *impossible* in safe
    // Rust:
    //
    //   fn dangle() -> &String {
    //       let s = String::from("oops");
    //       &s         // ❌ s is dropped at end of function; ref would dangle
    //   }
    //
    // Python doesn't have this bug because the GC keeps `s` alive for as
    // long as someone refers to it. Rust solves it differently: at compile
    // time, by refusing to compile the broken code.
}

// =================== EXERCISES ===================
//
// 1. The function below DOES NOT COMPILE. Fix it WITHOUT using `.clone()`.
//    (Hint: who needs to own `name`? The caller still wants to print it.
//    Should `greet` take ownership, or just borrow?)
//
//    fn greet(name: String) -> String { format!("hi, {name}") }
//    fn main_ex() {
//        let n = String::from("Ada");
//        let g = greet(n);
//        println!("{n} -> {g}");   // <-- "borrow of moved value: n"
//    }
//
//    The Python version "just works" because Python doesn't move. The
//    Rust idiom is to take `&str` instead of `String` when you only need
//    to read.
//
// 2. Write `fn longest_word(s: &str) -> &str` returning the longest
//    whitespace-separated word. Return a slice — no allocation.
//    Python: max(s.split(), key=len)
//
// 3. Implement `fn append_exclaim(s: &mut String)` and call it twice in a
//    row. Then break it: try to call it while a shared `&s` is alive. Read
//    the error message — note it points to the exact lines.
//
// 4. (Stretch) Write a struct that holds a string slice:
//        struct Excerpt<'a> { text: &'a str }
//    and a method `fn first_sentence(&self) -> &str`. The lifetime
//    annotation `'a` is mandatory here — the compiler needs to know that
//    the Excerpt can't outlive the string it refers to.
//
//    This is a *legitimately new concept* with no Python analog. If it
//    doesn't click on the first read, that's normal — most tutorial-level
//    Rust code never needs explicit lifetimes thanks to elision.
