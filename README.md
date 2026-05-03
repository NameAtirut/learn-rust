# Rust in a Day: Basics + Concurrency (for Python developers)

A practical, example-driven curriculum that takes a **Python programmer** from
zero Rust to confidently writing concurrent Rust in **8 focused hours**.
Every module includes side-by-side Python ↔ Rust comparisons so your existing
intuition does the heavy lifting.

## Who this is for

You write Python every day (Django/FastAPI/data tooling/scripts). You've heard
Rust is fast and "memory-safe", but you've bounced off the borrow checker
once already. This curriculum meets you where you are: each concept starts
with "here's how you'd do it in Python" before showing the Rust version.

## How to use this repo

1. Install Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
2. Clone, then for each hour:
   ```
   cargo run --example h1_basics
   ```
3. Read the file end-to-end **before** running. Every module starts with a
   "Python ↔ Rust at a glance" cheat sheet. Then run, then do the
   `// EXERCISE:` blocks at the bottom. Don't skip exercises — the borrow
   checker only teaches you when it yells at you.

## Schedule

| Hour | Topic                                              | Python analog                      | File                       |
|------|----------------------------------------------------|------------------------------------|----------------------------|
| 1    | Tooling, syntax, variables, control flow           | basic syntax, `for`/`while`/`if`   | `h1_basics.rs`             |
| 2    | Ownership, borrowing, lifetimes                    | references, GC, mutability         | `h2_ownership.rs`          |
| 3    | Structs, enums, pattern matching, `Option`/`Result` | dataclasses, `match`, `None`, exceptions | `h3_types.rs`        |
| 4    | Collections, iterators, error handling             | `list`/`dict`/`set`, comprehensions, try/except | `h4_collections.rs` |
| 5    | Traits, generics, closures                         | `Protocol`, generics, lambdas      | `h5_traits.rs`             |
| 6    | Threads, `Send`/`Sync`, `Arc`, `Mutex`             | `threading`, GIL, `Lock`           | `h6_threads.rs`            |
| 7    | Channels, scoped threads, data parallelism         | `queue.Queue`, `multiprocessing.Pool` | `h7_channels.rs`        |
| 8    | `async`/`await`, Tokio, capstone                   | `asyncio`, `aiohttp`               | `h8_async.rs`              |

## The Big Three "wait, what?" moments for Python users

1. **Variables are immutable by default.** `let x = 5;` is more like Python's
   `x: Final[int] = 5`. You opt into mutation with `let mut x = 5;`.
2. **No `None` — but there's `Option<T>`.** Same idea, but the type system
   *forces* you to handle the `None` case before you can use the value.
   Goodbye `AttributeError: 'NoneType' object has no attribute …`.
3. **Ownership replaces the GC.** Every value has exactly one owner; when
   the owner goes out of scope, the value is freed. There is no garbage
   collector. The compiler enforces this at compile time, which is also
   what makes Rust thread-safe by default.

## Learning principles applied

- **One concept per hour.** Each lesson builds on the previous one.
- **Compile-driven learning.** Every example deliberately includes commented
  code that *won't* compile, so you can uncomment and read the error.
  Reading errors is the #1 Rust skill, and Rust's errors are unusually
  good — they often suggest the fix.
- **Concurrency on a foundation.** We do not touch threads until ownership
  is solid. The borrow checker is what makes Rust's concurrency safe, and
  it has to land first.
- **Real APIs only.** Every concurrency primitive shown (`std::thread`,
  `std::sync`, `mpsc`, `rayon`, `tokio`) is what you would actually use in
  production.

## Suggested timing for a single-day sprint

- 0:00–1:00 Hour 1
- 1:00–2:00 Hour 2 (the hardest hour — don't rush it)
- 2:00–2:15 ☕ break
- 2:15–3:15 Hour 3
- 3:15–4:15 Hour 4
- 4:15–5:00 🍴 lunch
- 5:00–6:00 Hour 5
- 6:00–7:00 Hour 6
- 7:00–8:00 Hour 7
- 8:00–9:00 Hour 8 + capstone

## After this day

- Read [The Rust Book](https://doc.rust-lang.org/book/) chapters 13, 15, 17, 19 for depth.
- Solve [Rustlings](https://github.com/rust-lang/rustlings) for muscle memory.
- Build something real: a CLI with `clap`, a web server with `axum`, a parser with `nom`.

## Capstone (Hour 8)

A concurrent web crawler that fetches a list of URLs, parses titles, and
respects a max-concurrency limit using a `tokio::sync::Semaphore`. ~60 lines.
This is the Rust equivalent of `asyncio.gather` with a `Semaphore` —
you'll recognize the shape, but the safety guarantees are stronger.
