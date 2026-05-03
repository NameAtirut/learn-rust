//! Hour 3 — Structs, enums, pattern matching, Option, Result.
//!
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │ Python ↔ Rust at a glance                                           │
//! ├─────────────────────────────────────────────────────────────────────┤
//! │ @dataclass class Point: x: f; y: f      │ struct Point { x: f64, y: f64 } │
//! │ p.x  / p.distance_to(q)                 │ p.x  / p.distance_to(&q)        │
//! │ class Shape(Enum): CIRCLE = ...         │ enum Shape { Circle{r:f64}, ... }│
//! │ match s: case Circle(r): ...            │ match s { Shape::Circle{r}=>...}│
//! │ Optional[T]  /  if x is None:           │ Option<T>  /  match x { None=>..}│
//! │ try: ... except FooError:               │ Result<T, FooError> + match     │
//! │ raise FooError("...")                   │ return Err(FooError::Bad);      │
//! └─────────────────────────────────────────────────────────────────────┘
//!
//! Two key differences from Python:
//!
//!   1. Rust enums can carry DATA per variant. They're "tagged unions" /
//!      "sum types". Closer to Python's `Union[A, B, C]` than to `Enum`.
//!   2. There is NO `None`-the-keyword and NO exceptions. The two values
//!      you'll see a thousand times today are:
//!         - Option<T> = Some(T) | None     (replaces `Optional[T]`)
//!         - Result<T, E> = Ok(T) | Err(E)  (replaces try/except)
//!      You CAN'T accidentally use a missing value. The compiler insists
//!      you handle the absent/error case before it'll let you peek inside.
//!
//! Run with: cargo run --example h3_types

fn main() {
    structs();
    enums_and_match();
    options();
    results();
}

// --- structs ---
//
// Python:
//   @dataclass
//   class Point:
//       x: float
//       y: float
//
// `derive(Debug)` is like adding `__repr__` automatically.
// `derive(Clone)` is like making the object copyable via .clone().
#[derive(Debug, Clone)]
struct Point {
    x: f64,
    y: f64,
}

impl Point {
    /// Associated function (Python's `@classmethod`-ish; called Point::new(...))
    fn new(x: f64, y: f64) -> Self {
        Self { x, y }                 // shorthand: { x: x, y: y }
    }

    /// Method. `&self` is sugar for `self: &Self` — like Python's `self`,
    /// except it's an explicit borrow (read-only here).
    fn distance_to(&self, other: &Point) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    /// Mutating method: takes &mut self. Python doesn't distinguish — every
    /// method can mutate. Rust forces you to be explicit, which catches
    /// "I didn't mean to mutate that" bugs.
    fn translate(&mut self, dx: f64, dy: f64) {
        self.x += dx;
        self.y += dy;
    }
}

fn structs() {
    let a = Point::new(0.0, 0.0);
    let mut b = Point::new(3.0, 4.0);
    println!("[structs] a={a:?} b={b:?}");          // {:?} uses Debug (like repr())
    println!("[structs] dist = {}", a.distance_to(&b));
    b.translate(1.0, 1.0);
    println!("[structs] moved b = {b:?}");

    // "Tuple structs" — fields by index. Useful for newtype pattern, e.g. to
    // distinguish meters from feet at the type level — something you'd do
    // in Python with `NewType("Meters", float)`.
    struct Meters(f64);
    let height = Meters(1.83);
    println!("[structs] height = {} m", height.0);
}

// --- enums: sum types, FAR more powerful than Python's Enum ---
//
// Python:
//   class Shape:
//       pass
//   @dataclass
//   class Circle(Shape): radius: float
//   @dataclass
//   class Rect(Shape): w: float; h: float
//   class Unit(Shape): pass
//
// Rust packs all that into one declaration:
#[derive(Debug)]
enum Shape {
    Circle { radius: f64 },         // struct-like variant
    Rect(f64, f64),                 // tuple-like variant
    Unit,                           // no data
}

impl Shape {
    fn area(&self) -> f64 {
        // Like Python 3.10 `match`, but EXHAUSTIVE — the compiler will
        // refuse to compile if you forget a case. (Try removing one!)
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

    // `if let` is sugar for matching just one variant — Python equivalent:
    //   if isinstance(s, Circle): radius = s.radius
    let s = Shape::Circle { radius: 1.0 };
    if let Shape::Circle { radius } = s {
        println!("[enums] it's a circle, r={radius}");
    }
}

// --- Option<T> = Some(T) | None ---
//
// Replaces `Optional[T]` in Python. The killer feature: you literally
// cannot use the inner value without unwrapping/matching, so you can't
// `AttributeError: 'NoneType' object has no attribute 'x'`.
fn options() {
    let nums = [1, 2, 3];
    let first: Option<&i32> = nums.first();
    let tenth: Option<&i32> = nums.get(10);     // returns None, NOT a panic
    println!("[option] first={first:?} tenth={tenth:?}");

    // Python: doubled = first * 2 if first is not None else None
    // Rust  : Option has combinators. `.map` is "do this if Some".
    let doubled: Option<i32> = first.map(|x| x * 2);
    let or_default: i32 = tenth.copied().unwrap_or(-1);
    println!("[option] doubled={doubled:?} or_default={or_default}");

    // `?` short-circuits on `None` (in functions returning Option).
    // Python equivalent: an early `return None` if any step is None.
    fn third_char(s: &str) -> Option<char> {
        let c = s.chars().nth(2)?;        // returns None if too short
        Some(c.to_ascii_uppercase())
    }
    println!("[option] third_char(\"rust\") = {:?}", third_char("rust"));
    println!("[option] third_char(\"hi\")   = {:?}", third_char("hi"));
}

// --- Result<T, E> = Ok(T) | Err(E) ---
//
// Replaces try/except. Errors are VALUES, not control flow.
//
// Python:
//   class ParseAgeError(Exception): pass
//   def parse_age(s) -> int:
//       n = int(s)               # raises ValueError
//       if n > 150: raise ParseAgeError("out of range")
//       return n
//
// Rust:
#[derive(Debug)]
enum ParseAgeError {
    NotANumber,
    OutOfRange,
}

fn parse_age(s: &str) -> Result<u8, ParseAgeError> {
    // `?` propagates the error — it's the Rust equivalent of "let the
    // exception bubble up". `.map_err` converts the underlying ParseIntError
    // into our error type.
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
    // `?` works for both Option and Result. It's the same idea as Python's
    // exception unwinding — but VISIBLE in the source. Every `?` is a
    // potential early-return, and you can grep for them.
}

// =================== EXERCISES ===================
//
// 1. Add a `Triangle { base: f64, height: f64 }` variant to `Shape` and
//    update `area`. The compiler will FORCE you to handle it in `match`.
//    This is "exhaustiveness checking" — it prevents the bug where you add
//    a new case and forget to handle it somewhere. Python's `match` does
//    NOT do this; mypy can be configured to, with effort.
//
// 2. Write `fn safe_div(a: f64, b: f64) -> Option<f64>` that returns None
//    when b == 0.0.
//    Python: def safe_div(a, b): return None if b == 0 else a / b
//
// 3. Write `fn parse_pair(s: &str) -> Result<(i32, i32), String>` that
//    parses "3,4" into (3, 4). Use `?` for both parses.
//    Python: parts = s.split(","); return int(parts[0]), int(parts[1])
//
// 4. Implement `Point::origin() -> Self` returning (0,0). This is an
//    associated function (no `&self`) — like a Python `@classmethod`.
//
// 5. (Stretch) Define `enum Tree { Leaf(i32), Node(Box<Tree>, Box<Tree>) }`
//    and a recursive `fn sum(&self) -> i32`. Why is `Box` required?
//    Answer: a struct can't directly contain itself — its size would be
//    infinite. `Box<T>` is a heap pointer with known size. In Python you
//    don't think about this because everything is already a heap pointer.
