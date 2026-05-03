//! Hour 1 — Tooling, syntax, variables, control flow.
//!
//! Goal: be comfortable writing small Rust programs by the end of this hour.
//! Run with:  cargo run --example h1_basics
//!
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │ Python ↔ Rust at a glance                                           │
//! ├─────────────────────────────────────────────────────────────────────┤
//! │ Python                          │ Rust                              │
//! │ ──────                          │ ────                              │
//! │ x = 5                           │ let x = 5;                        │
//! │ x = 6  (mutates same name)      │ let mut x = 5; x = 6;             │
//! │ # type hint optional            │ let x: i32 = 5;  // strong, static│
//! │ "hello"                         │ "hello"     (a &str slice)        │
//! │ list("hello")                   │ String::from("hello")             │
//! │ if/elif/else                    │ if/else if/else                   │
//! │ for x in range(5):              │ for x in 0..5 {}                  │
//! │ def f(a, b): return a + b       │ fn f(a: i32, b: i32) -> i32 {a+b} │
//! │ pass (no-op)                    │ {}  (empty block)                 │
//! └─────────────────────────────────────────────────────────────────────┘
//!
//! Big mindset shift from Python:
//!   - Variables are IMMUTABLE by default. `let mut` to opt into mutation.
//!     This is closer to `x: Final = 5` in Python — but enforced.
//!   - Types are static. The compiler infers them most of the time, like
//!     mypy with `--strict` turned all the way up.
//!   - Almost everything is an EXPRESSION (returns a value). Even `if` and
//!     blocks `{ ... }` evaluate to a value.

fn main() {
    // --- variables: immutable by default ---
    // Python: x = 5
    let x = 5;            // type inferred as i32 (32-bit signed integer)
    let y: i64 = 10;      // explicit type annotation
    // x = 6;             // ❌ error: cannot assign twice. Uncomment to see it.
    //                    //    In Python this would just rebind. In Rust, no.
    let mut z = 0;        // `mut` opts into mutation (Python's default)
    z += 1;
    println!("x={x}, y={y}, z={z}");   // f-string-like; `{name}` reads vars

    // --- shadowing: NOT the same as Python rebinding ---
    // Shadowing creates a NEW variable that hides the old one. The old one
    // is still there, just inaccessible. You can even change the type:
    let n = "42";                       // n: &str (a string slice)
    let n: i32 = n.parse().expect("not a number");  // new n: i32
    println!("shadowed n={n}");
    // In Python, `n = "42"` then `n = int(n)` does the same thing
    // dynamically. Rust does it through the type system instead.

    // --- primitive types you'll meet today ---
    // Python has one int type and one float. Rust has many — pick by
    // size and signedness. Default to i32 (ints) and f64 (floats).
    let _signed: i32 = -1;
    let _unsigned: u64 = 1;       // unsigned — can't be negative
    let _float: f64 = 3.14;
    let _boolean: bool = true;
    let _char: char = '🦀';        // a single Unicode scalar value (4 bytes)
                                   // NOTE: Python has no char — chars are
                                   // 1-length strings. Rust char is its own type.
    let _tuple: (i32, &str) = (1, "a");      // like Python tuple, but typed
    let _array: [u8; 3] = [1, 2, 3];         // FIXED size on the stack
                                              // Python list ≈ Rust Vec (Hour 4)

    // --- strings: TWO types, this trips up Python users ---
    // Python has one `str`. Rust has:
    //   &str   — a borrowed view into string data (often a literal)
    //            Like a read-only memoryview into bytes.
    //   String — an owned, growable, heap-allocated UTF-8 buffer.
    //            Like Python's str + the mutability of bytearray.
    // You can convert: String::from(&str), or s.to_string()
    let greeting: &str = "hello";                // string literal
    let mut owned: String = String::from(greeting);
    owned.push_str(", world");                    // mutate in place
    println!("{owned}");

    // --- expressions, not statements ---
    // In Python, `if` is a statement; you'd use `x if cond else y`.
    // In Rust, `if` is an expression — and so are blocks. Note: NO trailing
    // semicolon on the line whose value the block returns.
    let abs = {
        let v: i32 = -7;
        if v < 0 { -v } else { v }   // <-- expression value of the block
    };
    println!("abs={abs}");

    // --- control flow ---
    // Python: for i in range(5):
    for i in 0..5 {              // 0..5 is a Range, exclusive end (like range)
        if i % 2 == 0 {
            println!("even: {i}");
        } else {
            println!("odd:  {i}");
        }
    }
    // Inclusive range: 0..=5  (Python equivalent: range(6))

    // `loop` is `while True` that returns a value via `break expr`.
    // Python has no direct equivalent — you'd use a flag and break.
    let first_square_over_50 = {
        let mut i = 0;
        loop {
            i += 1;
            if i * i > 50 {
                break i * i;       // returns this value from the loop
            }
        }
    };
    println!("first square > 50: {first_square_over_50}");

    // `match` is the workhorse. Python 3.10+ has `match`; Rust's is similar
    // but EXHAUSTIVE — the compiler refuses to let you forget a case.
    let label = match z {
        0 => "zero",
        1 | 2 => "one or two",
        3..=10 => "small",
        _ => "big",                 // _ is "anything else" (like Python's case _)
    };
    println!("label of z={z}: {label}");

    // --- functions ---
    // Python: def add(a, b): return a + b
    // Rust  : fn add(a: i32, b: i32) -> i32 { a + b }
    println!("add(2,3) = {}", add(2, 3));
    println!("fizzbuzz(15) = {}", fizzbuzz(15));
}

/// A function: parameters are typed, return type after `->`.
/// The body is an expression — no `return` keyword needed for the final
/// value (just no semicolon). You CAN use `return x;` for early returns.
fn add(a: i32, b: i32) -> i32 {
    a + b           // no semicolon = "this is what we return"
}

fn fizzbuzz(n: u32) -> String {
    // Python: "FizzBuzz" if n%15==0 else "Fizz" if n%3==0 else ...
    // Rust idiom: match on a tuple of remainders.
    match (n % 3, n % 5) {
        (0, 0) => "FizzBuzz".to_string(),
        (0, _) => "Fizz".to_string(),
        (_, 0) => "Buzz".to_string(),
        _ => n.to_string(),
    }
}

// =================== EXERCISES ===================
// Do these BEFORE moving to Hour 2.
//
// 1. Add a function `factorial(n: u32) -> u64`. Use a `for` loop.
//    Python equivalent: math.factorial(n) or a for-loop with accumulator.
//    Call it from main and print factorial(20).
//
// 2. Convert `factorial` to use `(1..=n).product()` instead.
//    This is the Rust answer to `math.prod(range(1, n+1))`. Iterators have
//    a `.product()` method — type inference figures out the result type
//    from your declared return type.
//
// 3. Write `fn is_prime(n: u32) -> bool` and print all primes under 50
//    using a `for` loop. (Python: `[n for n in range(2,50) if all(n%i for i in range(2,int(n**.5)+1))]`)
//
// 4. Uncomment the `// x = 6;` line at the top, run `cargo check`, read the
//    error. *Reading errors carefully is the #1 Rust skill.* Re-comment it.
//
// 5. (Stretch) Write `fn classify(c: char) -> &'static str` returning
//    "digit", "letter", "whitespace", or "other". Use char methods like
//    `c.is_ascii_digit()`, `c.is_alphabetic()`, `c.is_whitespace()`.
//    (Python equivalent: c.isdigit() / c.isalpha() / c.isspace().)
