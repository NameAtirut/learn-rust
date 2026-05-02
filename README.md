# Rust in a Day: Basics + Concurrency

A practical, example-driven curriculum that takes a programmer from zero Rust
to confidently writing concurrent Rust in **8 focused hours**. Each hour is a
runnable, commented program you read, run, then modify.

## Who this is for

You already program in another language (Python, Go, Java, TS, C++…). You
want to (a) be productive in Rust quickly and (b) understand what makes
Rust's concurrency story unique.

## How to use this repo

1. Install Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
2. Clone, then for each hour:
   ```
   cargo run --example h1_basics
   ```
3. Read the file end-to-end **before** running. Then run, then do the
   `// EXERCISE:` blocks at the bottom of each file. Don't skip exercises —
   the borrow checker only teaches you when it yells at you.

## Schedule

| Hour | Topic                                            | File                       |
|------|--------------------------------------------------|----------------------------|
| 1    | Tooling, syntax, variables, control flow         | `h1_basics.rs`             |
| 2    | Ownership, borrowing, lifetimes                  | `h2_ownership.rs`          |
| 3    | Structs, enums, pattern matching, `Option`/`Result` | `h3_types.rs`           |
| 4    | Collections, iterators, error handling           | `h4_collections.rs`        |
| 5    | Traits, generics, closures                       | `h5_traits.rs`             |
| 6    | Threads, `Send`/`Sync`, `Arc`, `Mutex`           | `h6_threads.rs`            |
| 7    | Channels, scoped threads, data parallelism       | `h7_channels.rs`           |
| 8    | `async`/`await`, Tokio, capstone                 | `h8_async.rs`              |

Each file is ~150 lines: ~80 lines of explanation/code, ~70 lines of
exercises and hints.

## Learning principles applied

- **One concept per hour.** You'll review the previous hour's exercise at
  the start of the next.
- **Compile-driven learning.** Every example deliberately includes commented
  code that *won't* compile, so you can uncomment and read the error.
- **Concurrency on a foundation.** We do not touch threads until ownership
  is solid. The borrow checker is what makes Rust's concurrency safe; it has
  to land first.
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
It exercises ownership, traits, error handling, async, and shared state.
