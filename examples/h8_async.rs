//! Hour 8 — async / await with Tokio + capstone.
//!
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │ Python ↔ Rust async at a glance                                     │
//! ├─────────────────────────────────────────────────────────────────────┤
//! │ Python                              │ Rust                          │
//! │ ──────                              │ ────                          │
//! │ async def f(): ...                  │ async fn f() { ... }          │
//! │ await coro                          │ future.await                  │
//! │ asyncio.run(main())                 │ #[tokio::main] async fn main()│
//! │ asyncio.create_task(f())            │ tokio::spawn(f())             │
//! │ asyncio.gather(a, b, c)             │ tokio::join!(a, b, c)         │
//! │ asyncio.wait(..., FIRST_COMPLETED)  │ tokio::select! { ... }        │
//! │ asyncio.Semaphore(n)                │ tokio::sync::Semaphore(n)     │
//! │ aiohttp.ClientSession()             │ reqwest::Client::new()        │
//! └─────────────────────────────────────────────────────────────────────┘
//!
//! Mental model — almost identical to Python's asyncio:
//!   - `async fn foo()` returns a `Future` (Python: a coroutine object).
//!     Calling it does NOTHING until you await/run it.
//!   - `.await` says "yield until this future is ready". The runtime parks
//!     the task and resumes it later. (Python: the same.)
//!   - `tokio::spawn` schedules a future as an independent task — like
//!     `asyncio.create_task`, but cheap to scale to hundreds of thousands.
//!   - You need a runtime (Tokio is the de facto choice; Python: the
//!     asyncio event loop).
//!
//! When to use what:
//!   - I/O concurrency (HTTP, DB, files): async (this hour).
//!   - CPU parallelism: threads + rayon (Hour 7).
//! Same advice as Python: don't put CPU-bound work in your async tasks.
//!
//! Run with: cargo run --example h8_async
//!
//! Capstone: a concurrent URL fetcher with bounded parallelism. It's the
//! Rust equivalent of an asyncio script using `aiohttp` + `Semaphore`. If
//! you've written that in Python, the shape will feel familiar.

use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use tokio::sync::Semaphore;

// `#[tokio::main]` is sugar for "set up the runtime and run main as a
// future". The Python equivalent is `asyncio.run(main())`.
#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    intro().await;
    spawn_and_join().await;
    sleep_concurrently().await?;
    capstone_fetch_titles().await?;
    Ok(())
}

// --- the simplest possible async ---
async fn intro() {
    async fn say(msg: &str) {
        println!("[intro] {msg}");
    }
    // Without `.await`, calling `say(...)` would just produce a Future and
    // do nothing. Same trap as Python: `say("hi")` would just create a
    // coroutine object and warn "coroutine was never awaited".
    say("hello from async").await;
}

// --- tokio::spawn for independent tasks ---
//
// Python: t = asyncio.create_task(f()); result = await t
async fn spawn_and_join() {
    let h = tokio::spawn(async {
        tokio::time::sleep(Duration::from_millis(10)).await;
        7u32
    });
    let v = h.await.expect("task panicked");
    println!("[spawn] task returned {v}");
}

// --- doing things concurrently with `join!` ---
//
// Python: a, b, c = await asyncio.gather(f(), g(), h())
async fn sleep_concurrently() -> Result<()> {
    let t = Instant::now();
    // tokio::join! polls the futures concurrently on the SAME task.
    // (For independent tasks across the runtime, use tokio::spawn.)
    let (a, b, c) = tokio::join!(
        tokio::time::sleep(Duration::from_millis(50)),
        tokio::time::sleep(Duration::from_millis(50)),
        tokio::time::sleep(Duration::from_millis(50)),
    );
    let _ = (a, b, c);
    println!("[join] 3x 50ms sleeps in {}ms", t.elapsed().as_millis());
    // Should print ~50, not ~150 — they ran concurrently. Same as Python.
    Ok(())
}

// --- capstone: concurrent fetch with bounded parallelism ---
//
// Goal: GET a list of URLs, extract <title>, print them. Cap parallelism at
// MAX_INFLIGHT requests using a Semaphore. Errors don't kill the whole
// program — each URL gets its own Result.
//
// This is the canonical Python async script:
//
//   async def fetch_title(session, url):
//       async with session.get(url, timeout=8) as r:
//           html = await r.text()
//       return extract_title(html)
//
//   async def main():
//       sem = asyncio.Semaphore(4)
//       async with aiohttp.ClientSession() as session:
//           async def fetch(url):
//               async with sem:
//                   return url, await fetch_title(session, url)
//           results = await asyncio.gather(*(fetch(u) for u in URLS),
//                                          return_exceptions=True)
//
// The Rust version below is conceptually identical.

const MAX_INFLIGHT: usize = 4;

async fn fetch_title(client: &reqwest::Client, url: &str) -> Result<String> {
    let body = client
        .get(url)
        .timeout(Duration::from_secs(8))
        .send()
        .await
        .with_context(|| format!("GET {url}"))?
        .text()
        .await
        .with_context(|| format!("read body of {url}"))?;
    let title = extract_title(&body).unwrap_or_else(|| "<no title>".into());
    Ok(title)
}

fn extract_title(html: &str) -> Option<String> {
    // Crude but dependency-free. In a real app, use the `scraper` crate.
    let lower = html.to_ascii_lowercase();
    let start = lower.find("<title")?;
    let after_open = start + html[start..].find('>')? + 1;
    let end_rel = lower[after_open..].find("</title>")?;
    Some(html[after_open..after_open + end_rel].trim().to_string())
}

async fn capstone_fetch_titles() -> Result<()> {
    let urls = vec![
        "https://www.rust-lang.org/",
        "https://doc.rust-lang.org/book/",
        "https://crates.io/",
        "https://tokio.rs/",
        "https://example.com/",
    ];

    // Like aiohttp.ClientSession — connection pool, configurable defaults.
    let client = reqwest::Client::builder()
        .user_agent("learn-rust/0.1")
        .build()?;

    // Arc<Semaphore> because we'll share it across tasks. (Python: just
    // refer to the Semaphore from each coroutine — no Arc needed because
    // GC.) The shape with Arc is the price you pay for compile-time
    // ownership — and the safety it gives you.
    let sem = Arc::new(Semaphore::new(MAX_INFLIGHT));

    let mut handles = Vec::with_capacity(urls.len());
    for url in urls {
        let permit = Arc::clone(&sem).acquire_owned().await?;  // wait if at limit
        let client = client.clone();                           // cheap (internal Arc)
        handles.push(tokio::spawn(async move {
            let result = fetch_title(&client, url).await;
            drop(permit);                                      // release the slot
            (url, result)
        }));
    }

    // Python: `for coro in asyncio.as_completed(tasks): ...`
    // Here we keep insertion order and just .await each handle.
    for h in handles {
        let (url, result) = h.await.expect("task panicked");
        match result {
            Ok(title) => println!("[capstone] {url} -> {title}"),
            Err(e)    => println!("[capstone] {url} FAILED: {e:#}"),
        }
    }
    Ok(())
}

// =================== EXERCISES ===================
//
// 1. Add a CLI: read URLs from stdin (one per line) instead of the
//    hardcoded list. Hint: `tokio::io::{AsyncBufReadExt, BufReader}` over
//    `tokio::io::stdin()`.
//    Python parallel: `for url in sys.stdin: ...` then await each.
//
// 2. Add retries: if `fetch_title` errors, retry up to 3 times with
//    exponential backoff (e.g. 200ms, 400ms, 800ms). Use
//    `tokio::time::sleep`.
//    Python: a wrapper coroutine with an async for-loop and asyncio.sleep.
//
// 3. Replace the `Vec<JoinHandle<...>>` collection pattern with
//    `futures::stream::iter(urls).map(...).buffer_unordered(MAX_INFLIGHT)`.
//    Add `futures = "0.3"` to Cargo.toml. Notice it eliminates the
//    explicit Semaphore.
//    Python parallel: this is closer to `aiostream` or rolling your own
//    bounded-gather helper.
//
// 4. (Stretch) Wrap the whole pipeline in a `tokio::select!` that races it
//    against a 30-second timeout. On timeout, print partial results.
//    Python: `asyncio.wait_for(main_pipeline(), timeout=30)`.
//
// 5. (Big stretch) Add a tiny axum web server that exposes
//    `GET /fetch?url=...`, runs the pipeline for one URL, and returns JSON
//    `{title, ms}`.
//    Python parallel: a FastAPI route doing the same.
