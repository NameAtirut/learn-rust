//! Hour 1 — Tooling, syntax, variables, control flow.
//!
//! Goal: be comfortable writing small Rust programs by the end of this hour.
//! Run with:  cargo run --example h1_basics
//!
//! Mental model: Rust is "C++ with a strict compiler and a friendly stdlib."
//! Most syntax will look familiar. The surprises are: immutability by default,
//! expressions-everywhere (almost everything returns a value), and the type
//! inference being stronger than you expect.

fn main() {
    // --- variables: immutable by default ---
    let x = 5;            // type inferred as i32
    let y: i64 = 10;      // explicit type
    // x = 6;             // ❌ error: cannot assign twice. Uncomment to see it.
    let mut z = 0;        // `mut` opts into mutation
    z += 1;
    println!("x={x}, y={y}, z={z}");

    // --- shadowing: not the same as mutation ---
    // You can re-declare with `let` and even change the type.
    let n = "42";
    let n: i32 = n.parse().expect("not a number");
    println!("shadowed n={n}");

    // --- primitive types you'll meet today ---
    let _signed: i32 = -1;
    let _unsigned: u64 = 1;
    let _float: f64 = 3.14;
    let _boolean: bool = true;
    let _char: char = '🦀';            // chars are 4 bytes (Unicode scalar values)
    let _tuple: (i32, &str) = (1, "a"); // fixed-size, mixed types
    let _array: [u8; 3] = [1, 2, 3];    // fixed-size, same type, stack-allocated

    // --- strings: two flavors ---
    // &str  : a borrowed string slice, often a literal, immutable
    // String: an owned, growable, heap-allocated UTF-8 buffer
    let greeting: &str = "hello";
    let mut owned: String = String::from(greeting);
    owned.push_str(", world");
    println!("{owned}");

    // --- expressions, not statements ---
    // Blocks return their last expression (no trailing `;`).
    let abs = {
        let v: i32 = -7;
        if v < 0 { -v } else { v }   // <-- expression, no semicolon
    };
    println!("abs={abs}");

    // --- control flow ---
    for i in 0..5 {              // 0..5 is a Range, exclusive end
        if i % 2 == 0 {
            println!("even: {i}");
        } else {
            println!("odd:  {i}");
        }
    }

    // `loop` returns a value via `break`.
    let first_square_over_50 = {
        let mut i = 0;
        loop {
            i += 1;
            if i * i > 50 {
                break i * i;
            }
        }
    };
    println!("first square > 50: {first_square_over_50}");

    // `match` is the workhorse — exhaustive pattern matching.
    let label = match z {
        0 => "zero",
        1 | 2 => "one or two",
        3..=10 => "small",
        _ => "big",
    };
    println!("label of z={z}: {label}");

    // --- functions ---
    println!("add(2,3) = {}", add(2, 3));
    println!("fizzbuzz(15) = {}", fizzbuzz(15));
}

/// A function: parameters are typed, return type after `->`.
/// The body is an expression — no `return` needed for the final value.
fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn fizzbuzz(n: u32) -> String {
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
//    Call it from main and print factorial(20).
//
// 2. Convert `factorial` to use `(1..=n).product()` instead. (Yes — iterators
//    have a `product` method. Type-inference will figure out the result type
//    if you annotate the return type.)
//
// 3. Write `fn is_prime(n: u32) -> bool` and print all primes under 50 using
//    a `for` loop and `if`.
//
// 4. Uncomment the `// x = 6;` line at the top, run `cargo check`, read the
//    error. *Reading errors carefully is the #1 Rust skill.* Re-comment it.
//
// 5. (Stretch) `match` exercise: write a function
//    `fn classify(c: char) -> &'static str` that returns "digit", "letter",
//    "whitespace", or "other". Use char methods like `c.is_ascii_digit()`.
