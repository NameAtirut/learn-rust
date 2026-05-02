//! Hour 3 — Structs, enums, pattern matching, Option, Result.
//!
//! Rust has no `null` and no exceptions. The two values you'll meet a
//! thousand times today are `Option<T>` (maybe a value) and `Result<T, E>`
//! (a value or an error). They're just enums — once you grok enums + match,
//! you have error handling.
//!
//! Run with: cargo run --example h3_types

fn main() {
    structs();
    enums_and_match();
    options();
    results();
}

// --- structs ---
#[derive(Debug, Clone)]   // derive printable + cloneable for free
struct Point {
    x: f64,
    y: f64,
}

impl Point {
    /// Associated function (think "static method"). Called `Point::new(...)`.
    fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Method: takes &self. `&self` is sugar for `self: &Self` — a borrow.
    fn distance_to(&self, other: &Point) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    /// Mutating method: takes &mut self.
    fn translate(&mut self, dx: f64, dy: f64) {
        self.x += dx;
        self.y += dy;
    }
}

fn structs() {
    let a = Point::new(0.0, 0.0);
    let mut b = Point::new(3.0, 4.0);
    println!("[structs] a={a:?} b={b:?}");
    println!("[structs] dist = {}", a.distance_to(&b));
    b.translate(1.0, 1.0);
    println!("[structs] moved b = {b:?}");

    // "Tuple structs" — fields by index. Useful for newtype patterns:
    struct Meters(f64);
    let height = Meters(1.83);
    println!("[structs] height = {} m", height.0);
}

// --- enums: sum types, far more powerful than C/Java enums ---
#[derive(Debug)]
enum Shape {
    Circle { radius: f64 },         // struct-like variant
    Rect(f64, f64),                 // tuple-like variant
    Unit,                           // no data
}

impl Shape {
    fn area(&self) -> f64 {
        match self {
            Shape::Circle { radius } => std::f64::consts::PI * radius * radius,
            Shape::Rect(w, h) => w * h,
            Shape::Unit => 0.0,
        }
    }
}

fn enums_and_match() {
    let shapes = [
        Shape::Circle { radius: 2.0 },
        Shape::Rect(3.0, 4.0),
        Shape::Unit,
    ];
    for s in &shapes {
        println!("[enums] {s:?} area = {:.3}", s.area());
    }

    // `if let` is sugar for matching just one variant:
    let s = Shape::Circle { radius: 1.0 };
    if let Shape::Circle { radius } = s {
        println!("[enums] it's a circle, r={radius}");
    }
}

// --- Option<T> = Some(T) | None ---
fn options() {
    let nums = [1, 2, 3];
    let first: Option<&i32> = nums.first();
    let tenth: Option<&i32> = nums.get(10);
    println!("[option] first={first:?} tenth={tenth:?}");

    // Chain combinators instead of nested if-let:
    let doubled: Option<i32> = first.map(|x| x * 2);
    let or_default: i32 = tenth.copied().unwrap_or(-1);
    println!("[option] doubled={doubled:?} or_default={or_default}");

    // `?` short-circuits on `None` (in functions returning Option).
    fn third_char(s: &str) -> Option<char> {
        let c = s.chars().nth(2)?;        // returns None if too short
        Some(c.to_ascii_uppercase())
    }
    println!("[option] third_char(\"rust\") = {:?}", third_char("rust"));
    println!("[option] third_char(\"hi\")   = {:?}", third_char("hi"));
}

// --- Result<T, E> = Ok(T) | Err(E) ---
#[derive(Debug)]
enum ParseAgeError {
    NotANumber,
    OutOfRange,
}

fn parse_age(s: &str) -> Result<u8, ParseAgeError> {
    let n: u32 = s.parse().map_err(|_| ParseAgeError::NotANumber)?;
    if n > 150 {
        Err(ParseAgeError::OutOfRange)
    } else {
        Ok(n as u8)
    }
}

fn results() {
    for s in ["29", "abc", "999"] {
        match parse_age(s) {
            Ok(age) => println!("[result] {s:?} -> age {age}"),
            Err(e)  => println!("[result] {s:?} -> error {e:?}"),
        }
    }
    // The `?` operator propagates errors up. Both Option and Result support it.
}

// =================== EXERCISES ===================
//
// 1. Add a `Triangle { base: f64, height: f64 }` variant to `Shape` and
//    update `area`. Note: the compiler will FORCE you to handle it in `match`.
//    This is "exhaustiveness checking" — embrace it.
//
// 2. Write `fn safe_div(a: f64, b: f64) -> Option<f64>` that returns None
//    when b == 0.0.
//
// 3. Write `fn parse_pair(s: &str) -> Result<(i32, i32), String>` that
//    parses "3,4" into (3, 4). Use `?` for both parses.
//
// 4. Implement a method `Point::origin() -> Self` returning (0,0). This is
//    an associated fn (no `&self`).
//
// 5. (Stretch) Define `enum Tree { Leaf(i32), Node(Box<Tree>, Box<Tree>) }`
//    and a recursive `fn sum(&self) -> i32`. Why is `Box` required?
//    (Answer: a struct can't directly contain itself — its size would be
//    infinite. Box is a heap pointer with a known size.)
